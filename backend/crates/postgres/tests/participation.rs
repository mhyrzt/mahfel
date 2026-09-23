use mahfel_application::ports::{
    ParticipantStore, ParticipationStore, RoomStore, SessionStore, StoreError, UserStore,
};
use mahfel_domain::{
    CorrectionReason, DisplayName, Participant, ParticipationInterval, Room, RoomMember, RoomRole,
    RoomTitle, RoomVisibility, Session, User, total_connected_seconds,
};
use mahfel_postgres::{
    ConnectionSettings, PoolSettings, PostgresParticipantStore, PostgresParticipationStore,
    PostgresRoomStore, PostgresSessionStore, PostgresUserStore, connect, migrate,
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
async fn participation_intervals_stay_consistent() -> Result<(), Box<dyn Error>> {
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

    let verification = verify_participation(&pool).await;
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

async fn verify_participation(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    let base = OffsetDateTime::now_utc()
        .replace_nanosecond(0)
        .expect("zero nanoseconds is valid");
    let at = |secs: i64| base + time::Duration::seconds(secs);

    let users = PostgresUserStore::new(pool.clone());
    let rooms = PostgresRoomStore::new(pool.clone());
    let sessions = PostgresSessionStore::new(pool.clone());
    let participants = PostgresParticipantStore::new(pool.clone());
    let participation = PostgresParticipationStore::new(pool.clone());

    let owner = User::create(
        DisplayName::new("Ada Lovelace".to_owned())?,
        mahfel_domain::EmailAddress::new("test@example.test".to_owned())?,
        mahfel_domain::Username::new("test-user".to_owned())?,
        base,
    );
    users.insert(&owner).await?;
    let member = User::create(
        DisplayName::new("Grace Hopper".to_owned())?,
        mahfel_domain::EmailAddress::new("test@example.test".to_owned())?,
        mahfel_domain::Username::new("test-user".to_owned())?,
        base,
    );
    users.insert(&member).await?;

    let room = Room::create(
        RoomTitle::new("Analytical Engines".to_owned())?,
        owner.id,
        RoomVisibility::InviteOnly,
        base,
    );
    rooms
        .create_with_manager(
            &room,
            &RoomMember::new(room.id, owner.id, RoomRole::Manager, base),
        )
        .await?;

    let mut session = Session::schedule(room.id, None, base);
    session.start(base)?;
    sessions.insert(&session).await?;

    let participant = Participant::join(session.id, member.id, base);
    participants.insert(&participant).await?;

    // Open the first stretch, then close it.
    let first = ParticipationInterval::open(participant.id, session.id, member.id, at(0));
    participation.open_interval(&first).await?;

    // A second open interval for the same participant must collide on the
    // partial unique index, so intervals cannot overlap.
    let racing = ParticipationInterval::open(participant.id, session.id, member.id, at(1));
    let conflict = participation.open_interval(&racing).await;
    assert!(
        matches!(conflict, Err(StoreError::Conflict(_))),
        "a second open interval must conflict, got {conflict:?}"
    );

    let closed = participation
        .close_open_interval(participant.id, at(60))
        .await?;
    assert_eq!(closed, Some(first.id));

    // Closing again is idempotent and does not move the end time.
    assert_eq!(
        participation
            .close_open_interval(participant.id, at(600))
            .await?,
        None
    );
    assert_eq!(
        participation.get(first.id).await?.expect("stored").ended_at,
        Some(at(60))
    );

    // A reconnect opens a new interval for the same durable participant.
    let second = ParticipationInterval::open(participant.id, session.id, member.id, at(120));
    participation.open_interval(&second).await?;

    let intervals = participation.list_for_participant(participant.id).await?;
    assert_eq!(intervals.len(), 2);
    assert_eq!(
        total_connected_seconds(&intervals, at(200), None),
        140,
        "60 connected + 80 still open"
    );

    // A correction supersedes the original and inserts a replacement in one
    // transaction, so the original row survives.
    let mut corrected = second.clone();
    let mut replacement =
        ParticipationInterval::open(participant.id, session.id, member.id, at(120));
    replacement.close(at(150))?;
    corrected.supersede(replacement.id, CorrectionReason::WrongEnd, at(180))?;
    participation
        .supersede_interval(&corrected, &replacement)
        .await?;

    // Superseding twice must be refused rather than recording a second
    // competing replacement.
    let again = participation
        .supersede_interval(&corrected, &replacement)
        .await;
    assert!(
        matches!(again, Err(StoreError::Conflict(_))),
        "a superseded interval cannot be superseded again, got {again:?}"
    );

    let intervals = participation.list_for_participant(participant.id).await?;
    assert_eq!(intervals.len(), 3, "history retains the superseded row");
    let superseded = participation.get(second.id).await?.expect("stored");
    assert_eq!(superseded.superseded_by, Some(replacement.id));
    assert_eq!(
        superseded.correction_reason,
        Some(CorrectionReason::WrongEnd)
    );
    assert!(
        !superseded.is_open(),
        "a superseded interval is no longer counted as open"
    );
    assert_eq!(
        total_connected_seconds(&intervals, at(200), None),
        90,
        "the superseded interval must not also count"
    );

    // The end-of-session sweep closes anything still open.
    let third = ParticipationInterval::open(participant.id, session.id, member.id, at(180));
    participation.open_interval(&third).await?;
    let swept = participation
        .close_open_intervals_for_session(session.id, at(300))
        .await?;
    assert_eq!(swept, 1, "only the open interval is closed");
    assert_eq!(
        participation
            .close_open_intervals_for_session(session.id, at(400))
            .await?,
        0,
        "the sweep is idempotent"
    );
    assert!(
        participation
            .list_for_session(session.id)
            .await?
            .iter()
            .all(|interval| !interval.is_open()),
        "no interval may remain open after the session ends"
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
