use mahfel_application::ports::{ParticipantStore, RoomStore, SessionStore, StoreError, UserStore};
use mahfel_domain::{
    DisplayName, Participant, ParticipantStatus, Room, RoomMember, RoomRole, RoomTitle,
    RoomVisibility, Session, SessionId, User,
};
use mahfel_postgres::{
    ConnectionSettings, PoolSettings, PostgresParticipantStore, PostgresRoomStore,
    PostgresSessionStore, PostgresUserStore, connect, migrate,
};
use sqlx::PgPool;
use std::{error::Error, time::Duration};
use testcontainers::{
    GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};
use time::OffsetDateTime;
use url::Url;
use uuid::Uuid;

fn test_pool_settings() -> PoolSettings {
    PoolSettings {
        min_connections: 1,
        max_connections: 2,
        acquire_timeout: Duration::from_secs(5),
        idle_timeout: Duration::from_secs(30),
        max_lifetime: Duration::from_secs(60),
    }
}

#[tokio::test]
async fn participants_are_unique_per_active_actor() -> Result<(), Box<dyn Error>> {
    if let Ok(admin_url) = std::env::var("MAHFEL_TEST_DATABASE_URL") {
        return verify_disposable_database(&admin_url).await;
    }

    if std::env::var("MAHFEL_TESTCONTAINERS").as_deref() != Ok("1") {
        eprintln!(
            "MAHFEL_TESTCONTAINERS is unset and MAHFEL_TEST_DATABASE_URL is unset; skipping PostgreSQL integration test"
        );
        return Ok(());
    }

    let postgres = GenericImage::new("postgres", "16-bookworm")
        .with_exposed_port(5432.tcp())
        .with_wait_for(WaitFor::message_on_stdout(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_DB", "postgres")
        .with_env_var("POSTGRES_USER", "mahfel")
        .with_env_var("POSTGRES_PASSWORD", "mahfel-test")
        .start()
        .await?;
    let port = postgres.get_host_port_ipv4(5432).await?;
    let admin_url = format!("postgres://mahfel:mahfel-test@127.0.0.1:{port}/postgres");
    wait_until_ready(&admin_url).await?;
    verify_disposable_database(&admin_url).await
}

/// Wait until the container accepts an authenticated connection.
///
/// The `ready to accept connections` log line is emitted before the auth
/// backend can complete a handshake, so connecting immediately can be reset by
/// the server. Probe with a real pool instead of sleeping a fixed amount.
async fn wait_until_ready(admin_url: &str) -> Result<(), Box<dyn Error>> {
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    loop {
        match PgPool::connect(admin_url).await {
            Ok(pool) => {
                pool.close().await;
                return Ok(());
            }
            Err(error) if std::time::Instant::now() < deadline => {
                eprintln!("postgres not ready yet: {error}");
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
            Err(error) => return Err(error.into()),
        }
    }
}

async fn verify_disposable_database(admin_url: &str) -> Result<(), Box<dyn Error>> {
    let admin_pool = PgPool::connect(admin_url).await?;
    let database_name = format!("mahfel_test_{}", Uuid::new_v4().simple());
    let create_database = format!("CREATE DATABASE \"{database_name}\" TEMPLATE template0");
    sqlx::raw_sql(sqlx::AssertSqlSafe(create_database))
        .execute(&admin_pool)
        .await?;

    let mut test_url = Url::parse(admin_url)?;
    test_url.set_path(&format!("/{database_name}"));
    let connection = connection_settings(&test_url)?;
    let pool = connect(&connection, test_pool_settings()).await?;
    migrate(&pool).await?;

    let verification = verify_participants(&pool).await;
    if !pool.is_closed() {
        pool.close().await;
    }

    let drop_database = format!("DROP DATABASE \"{database_name}\" WITH (FORCE)");
    let cleanup = sqlx::raw_sql(sqlx::AssertSqlSafe(drop_database))
        .execute(&admin_pool)
        .await;
    admin_pool.close().await;

    verification?;
    cleanup?;
    Ok(())
}

async fn verify_participants(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    let now = OffsetDateTime::now_utc()
        .replace_nanosecond(0)
        .expect("zero nanoseconds is valid");
    let users = PostgresUserStore::new(pool.clone());
    let rooms = PostgresRoomStore::new(pool.clone());
    let sessions = PostgresSessionStore::new(pool.clone());
    let participants = PostgresParticipantStore::new(pool.clone());

    let owner = User::create(
        DisplayName::new("Ada Lovelace".to_owned())?,
        mahfel_domain::EmailAddress::new("test@example.test".to_owned())?,
        mahfel_domain::Username::new("test-user".to_owned())?,
        now,
    );
    users.insert(&owner).await?;
    let member = User::create(
        DisplayName::new("Grace Hopper".to_owned())?,
        mahfel_domain::EmailAddress::new("test@example.test".to_owned())?,
        mahfel_domain::Username::new("test-user".to_owned())?,
        now,
    );
    users.insert(&member).await?;

    let room = Room::create(
        RoomTitle::new("Analytical Engines".to_owned())?,
        owner.id,
        RoomVisibility::InviteOnly,
        now,
    );
    rooms
        .create_with_manager(
            &room,
            &RoomMember::new(room.id, owner.id, RoomRole::Manager, now),
        )
        .await?;

    let mut session = Session::schedule(room.id, None, now);
    session.start(now)?;
    sessions.insert(&session).await?;

    let first = Participant::join(session.id, member.id, now);
    participants.insert(&first).await?;
    assert_eq!(
        participants.get_active(session.id, member.id).await?,
        Some(first.clone())
    );

    // A racing join must collide on the partial unique index.
    let racing = Participant::join(session.id, member.id, now + Duration::from_secs(1));
    let conflict = participants.insert(&racing).await;
    assert!(
        matches!(conflict, Err(StoreError::Conflict(_))),
        "a second active participant must conflict, got {conflict:?}"
    );

    let mut first = first;
    first.leave(now + Duration::from_secs(2))?;
    participants.save(&first).await?;
    assert_eq!(participants.get_active(session.id, member.id).await?, None);

    let rejoined = Participant::join(session.id, member.id, now + Duration::from_secs(3));
    participants.insert(&rejoined).await?;
    assert_ne!(rejoined.id, first.id);

    // History is append-only: the left row is retained alongside the new one.
    let all = participants.list_for_session(session.id).await?;
    assert_eq!(all.len(), 2, "both participations must be retained");
    assert_eq!(all[0].status, ParticipantStatus::Left);
    assert_eq!(all[1].status, ParticipantStatus::Active);

    // A second active row is allowed for a different actor.
    let other = Participant::join(session.id, owner.id, now);
    participants.insert(&other).await?;
    assert_eq!(participants.list_for_session(session.id).await?.len(), 3);
    assert!(
        participants
            .list_for_session(SessionId::new())
            .await?
            .is_empty(),
        "another session has no participants"
    );

    let orphan = Participant::join(SessionId::new(), member.id, now);
    assert!(
        participants.insert(&orphan).await.is_err(),
        "a participant must reference a real session"
    );

    Ok(())
}

fn connection_settings(url: &Url) -> Result<ConnectionSettings, Box<dyn Error>> {
    Ok(ConnectionSettings {
        host: url
            .host_str()
            .ok_or("test database URL is missing a host")?
            .to_owned(),
        port: url.port().unwrap_or(5432),
        user: url.username().to_owned(),
        password: url.password().unwrap_or_default().to_owned(),
        name: url.path().trim_start_matches('/').to_owned(),
    })
}
