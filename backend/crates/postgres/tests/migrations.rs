use mahfel_application::ports::{RoomStore, SessionStore, StoreError, UserStore};
use mahfel_domain::{
    ClassroomSession, DisplayName, Room, RoomMember, RoomRole, RoomTitle, RoomVisibility, User,
    UserId,
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
async fn migrations_provision_a_disposable_database() -> Result<(), Box<dyn Error>> {
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
    verify_disposable_database(&admin_url).await
}

async fn verify_disposable_database(admin_url: &str) -> Result<(), Box<dyn Error>> {
    let admin_pool = PgPool::connect(admin_url).await?;
    let database_name = format!("mahfel_test_{}", Uuid::new_v4().simple());
    let create_database = format!("CREATE DATABASE \"{database_name}\" TEMPLATE template0");
    // The interpolated identifier is generated exclusively from a UUID.
    sqlx::raw_sql(sqlx::AssertSqlSafe(create_database))
        .execute(&admin_pool)
        .await?;

    let mut test_url = Url::parse(admin_url)?;
    test_url.set_path(&format!("/{database_name}"));
    let connection = connection_settings(&test_url)?;
    let test_pool = connect(&connection, test_pool_settings()).await?;
    let verification = verify_migrations_and_stores(&test_pool, &connection).await;
    if !test_pool.is_closed() {
        test_pool.close().await;
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

async fn verify_migrations_and_stores(
    pool: &PgPool,
    connection: &ConnectionSettings,
) -> Result<(), Box<dyn Error>> {
    migrate(pool).await?;

    // Re-running migrations on an already-migrated database must be a no-op.
    // The connection pins `search_path=public` because the database role and
    // the `mahfel` schema share a name; otherwise `_sqlx_migrations` resolves
    // to a different schema before and after migration 0001 creates `mahfel`.
    migrate(pool).await?;

    let migration_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(pool)
        .await?;
    assert_eq!(migration_count, 3, "each migration must be recorded once");

    let migration_tables = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM pg_class WHERE relname = '_sqlx_migrations'",
    )
    .fetch_one(pool)
    .await?;
    assert_eq!(
        migration_tables, 1,
        "migration bookkeeping must use a single table"
    );

    let namespace_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'mahfel')",
    )
    .fetch_one(pool)
    .await?;
    assert!(
        namespace_exists,
        "migration must provision the Mahfel schema"
    );

    // PostgreSQL keeps microseconds; align the in-memory clock so round-trip
    // equality assertions compare the same instant.
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
    let manager = RoomMember::new(room.id, user.id, RoomRole::Manager, now);
    rooms.create_with_manager(&room, &manager).await?;

    let session = ClassroomSession::schedule(room.id, None, now);
    sessions.insert(&session).await?;

    assert_eq!(
        rooms.get_member(room.id, user.id).await?,
        Some(manager.clone())
    );
    assert_eq!(rooms.list_members(room.id).await?, vec![manager.clone()]);
    assert_eq!(
        rooms.list_for_member(user.id, 10, 0).await?,
        vec![room.clone()]
    );

    // A missing referenced user must surface as a typed adapter error rather
    // than a generic storage failure.
    let missing = rooms
        .add_member(&RoomMember::new(
            room.id,
            UserId::new(),
            RoomRole::Member,
            now,
        ))
        .await;
    assert!(
        matches!(missing, Err(StoreError::MissingReference(_))),
        "unknown user must map to MissingReference, got {missing:?}"
    );

    // The last manager cannot be removed; a co-manager can.
    let co_manager = User::create(DisplayName::new("Grace Hopper".to_owned())?, now);
    users.insert(&co_manager).await?;
    rooms
        .add_member(&RoomMember::new(
            room.id,
            co_manager.id,
            RoomRole::Manager,
            now,
        ))
        .await?;
    assert!(
        rooms.remove_member(room.id, co_manager.id).await?,
        "removing a co-manager must succeed"
    );
    assert!(
        !rooms.remove_member(room.id, user.id).await?,
        "the last manager must not be removed"
    );

    pool.close().await;
    let restarted_pool = connect(connection, test_pool_settings()).await?;
    let restarted_users = PostgresUserStore::new(restarted_pool.clone());
    let restarted_rooms = PostgresRoomStore::new(restarted_pool.clone());
    let restarted_sessions = PostgresSessionStore::new(restarted_pool.clone());

    assert_eq!(restarted_users.get(user.id).await?, Some(user.clone()));
    assert_eq!(restarted_rooms.get(room.id).await?, Some(room.clone()));
    assert_eq!(restarted_sessions.get(session.id).await?, Some(session));
    assert_eq!(
        restarted_rooms.get_member(room.id, user.id).await?,
        Some(manager.clone())
    );
    assert_eq!(
        restarted_rooms.list_for_member(user.id, 10, 0).await?,
        vec![room]
    );
    restarted_pool.close().await;
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
