use mahfel_application::ports::{RoomStore, SessionStore, StoreError, UserStore};
use mahfel_domain::{
    DisplayName, Room, RoomId, RoomMember, RoomRole, RoomTitle, RoomVisibility, Session,
    SessionStatus, User,
};
use mahfel_postgres::{
    ConnectionSettings, PoolSettings, PostgresRoomStore, PostgresSessionStore, PostgresUserStore,
    connect, migrate,
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
async fn sessions_enforce_one_live_per_room() -> Result<(), Box<dyn Error>> {
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

    let verification = verify_sessions(&pool).await;
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

async fn verify_sessions(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    let now = OffsetDateTime::now_utc()
        .replace_nanosecond(0)
        .expect("zero nanoseconds is valid");
    let users = PostgresUserStore::new(pool.clone());
    let rooms = PostgresRoomStore::new(pool.clone());
    let sessions = PostgresSessionStore::new(pool.clone());

    let user = User::create(DisplayName::new("Ada Lovelace".to_owned())?, now);
    users.insert(&user).await?;
    let room = Room::create(
        RoomTitle::new("Analytical Engines".to_owned())?,
        user.id,
        RoomVisibility::InviteOnly,
        now,
    );
    rooms
        .create_with_manager(
            &room,
            &RoomMember::new(room.id, user.id, RoomRole::Manager, now),
        )
        .await?;

    // Two scheduled sessions for one room.
    let first = Session::schedule(room.id, None, now);
    sessions.insert(&first).await?;
    let second = Session::schedule(room.id, None, now + Duration::from_secs(1));
    sessions.insert(&second).await?;

    // Listing is scoped to the room and returns newest first.
    let listed = sessions.list_for_room(room.id).await?;
    assert_eq!(listed, vec![second.clone(), first.clone()]);
    assert!(
        sessions.list_for_room(RoomId::new()).await?.is_empty(),
        "another room has no sessions"
    );

    // Only one session may be live at a time; the losing write conflicts.
    let mut first = first;
    first.start(now + Duration::from_secs(1))?;
    sessions.save(&first).await?;

    let mut second = second;
    second.start(now + Duration::from_secs(2))?;
    let conflict = sessions.save(&second).await;
    assert!(
        matches!(conflict, Err(StoreError::Conflict(_))),
        "a second live session must conflict, got {conflict:?}"
    );

    // The failed write left the second session scheduled.
    let stored = sessions
        .get(second.id)
        .await?
        .expect("second session row is retained");
    assert_eq!(stored.status, SessionStatus::Scheduled);
    let live = sessions
        .list_for_room(room.id)
        .await?
        .into_iter()
        .filter(|session| session.status == SessionStatus::Live)
        .count();
    assert_eq!(live, 1, "exactly one live session survives the race");

    // Concurrent starts for one room: exactly one write wins.
    let race_room = Room::create(
        RoomTitle::new("Concurrency".to_owned())?,
        user.id,
        RoomVisibility::InviteOnly,
        now,
    );
    rooms
        .create_with_manager(
            &race_room,
            &RoomMember::new(race_room.id, user.id, RoomRole::Manager, now),
        )
        .await?;
    let mut left = Session::schedule(race_room.id, None, now);
    let mut right = Session::schedule(race_room.id, None, now);
    sessions.insert(&left).await?;
    sessions.insert(&right).await?;
    left.start(now + Duration::from_secs(1))?;
    right.start(now + Duration::from_secs(1))?;
    let (left_result, right_result) = tokio::join!(sessions.save(&left), sessions.save(&right));
    let outcomes = [left_result, right_result];
    assert_eq!(
        outcomes.iter().filter(|result| result.is_ok()).count(),
        1,
        "exactly one concurrent start may win, got {outcomes:?}"
    );
    assert_eq!(
        outcomes
            .iter()
            .filter(|result| matches!(result, Err(StoreError::Conflict(_))))
            .count(),
        1,
        "the losing concurrent start must conflict, got {outcomes:?}"
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
