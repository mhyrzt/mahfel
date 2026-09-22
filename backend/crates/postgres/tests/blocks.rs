use mahfel_application::ports::{BlockStore, RoomStore, SessionStore, StoreError, UserStore};
use mahfel_domain::{
    Block, BlockScope, DisplayName, Room, RoomMember, RoomRole, RoomTitle, RoomVisibility, Session,
    User,
};
use mahfel_postgres::{
    ConnectionSettings, PoolSettings, PostgresBlockStore, PostgresRoomStore, PostgresSessionStore,
    PostgresUserStore, connect, migrate,
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
async fn blocks_are_unique_while_active_and_expire_on_their_own() -> Result<(), Box<dyn Error>> {
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

    let verification = verify_blocks(&pool).await;
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

async fn verify_blocks(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    let users = PostgresUserStore::new(pool.clone());
    let rooms = PostgresRoomStore::new(pool.clone());
    let sessions = PostgresSessionStore::new(pool.clone());
    let blocks = PostgresBlockStore::new(pool.clone());

    let base = OffsetDateTime::now_utc();
    let manager = User::create(DisplayName::new("Grace Hopper".to_owned())?, base);
    let member = User::create(DisplayName::new("Ada Lovelace".to_owned())?, base);
    users.insert(&manager).await?;
    users.insert(&member).await?;

    let mut room = Room::create(
        RoomTitle::new("Analytical Engines".to_owned())?,
        manager.id,
        RoomVisibility::InviteOnly,
        base,
    );
    room.admission = mahfel_domain::AdmissionPolicy::Open;
    rooms
        .create_with_manager(
            &room,
            &RoomMember::new(room.id, manager.id, RoomRole::Manager, base),
        )
        .await?;

    let mut session = Session::schedule(room.id, None, base);
    session.start(base)?;
    sessions.insert(&session).await?;

    // An active room block is durable.
    let room_block = Block::new(
        BlockScope::Room(room.id),
        member.id,
        manager.id,
        Some("off-topic".to_owned()),
        None,
        base,
    )?;
    blocks.insert(&room_block).await?;
    let active = blocks
        .get_active(BlockScope::Room(room.id), member.id)
        .await?
        .expect("active block");
    assert!(active.is_active_at(base));
    assert_eq!(active.reason.as_deref(), Some("off-topic"));

    // A second active block for the same target is refused by the partial unique
    // index, which is what makes re-blocking an extension rather than a stack.
    let duplicate = Block::new(
        BlockScope::Room(room.id),
        member.id,
        manager.id,
        None,
        None,
        base,
    )?;
    assert!(
        matches!(
            blocks.insert(&duplicate).await,
            Err(StoreError::Conflict(_))
        ),
        "at most one active block per target"
    );

    // Extending replaces the expiry rather than inserting.
    assert!(
        blocks
            .extend(room_block.id, Some(base + Duration::from_secs(3600)))
            .await?
    );
    let extended = blocks
        .get_active(BlockScope::Room(room.id), member.id)
        .await?
        .expect("extended block");
    assert!(extended.expires_at.is_some());

    // A room block and a session block are independent facts, so both can be
    // active at once and lifting one leaves the other.
    let session_block = Block::new(
        BlockScope::Session(session.id),
        member.id,
        manager.id,
        None,
        None,
        base,
    )?;
    blocks.insert(&session_block).await?;
    assert!(blocks.lift(room_block.id, manager.id, base).await?);
    assert!(
        blocks
            .get_active(BlockScope::Room(room.id), member.id)
            .await?
            .is_none(),
        "the lifted room block is no longer active"
    );
    assert!(
        blocks
            .get_active(BlockScope::Session(session.id), member.id)
            .await?
            .is_some(),
        "lifting the room block must not disturb the session block"
    );

    // A lapsed block is inactive without being lifted, so a new block does not
    // conflict with it. The expiry is in the past, because the column holds an
    // instant and not a duration.
    let lapsed = Block::new(
        BlockScope::Room(room.id),
        member.id,
        manager.id,
        None,
        Some(base + Duration::from_secs(1)),
        base,
    )?;
    blocks.insert(&lapsed).await?;
    // Wait for it to lapse, so the assertion is about expiry rather than about
    // the clock happening to be on one side of it.
    tokio::time::sleep(Duration::from_secs(2)).await;
    assert!(
        blocks
            .get_active(BlockScope::Room(room.id), member.id)
            .await?
            .is_none(),
        "an expired block is not active"
    );
    let fresh = Block::new(
        BlockScope::Room(room.id),
        member.id,
        manager.id,
        None,
        None,
        OffsetDateTime::now_utc(),
    )?;
    blocks
        .insert(&fresh)
        .await
        .expect("a lapsed block must not conflict with a new one");

    // The scope CHECK refuses a row that names both targets or neither.
    let both = sqlx::query(
        "INSERT INTO mahfel.blocks \
         (id, room_id, session_id, actor_id, blocked_by, blocked_at) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(Uuid::new_v4())
    .bind(room.id.as_uuid())
    .bind(session.id.as_uuid())
    .bind(member.id.as_uuid())
    .bind(manager.id.as_uuid())
    .bind(base)
    .execute(pool)
    .await;
    assert!(both.is_err(), "a block must not name both scopes");

    let neither = sqlx::query(
        "INSERT INTO mahfel.blocks \
         (id, room_id, session_id, actor_id, blocked_by, blocked_at) \
         VALUES ($1, NULL, NULL, $2, $3, $4)",
    )
    .bind(Uuid::new_v4())
    .bind(member.id.as_uuid())
    .bind(manager.id.as_uuid())
    .bind(base)
    .execute(pool)
    .await;
    assert!(neither.is_err(), "a block must name a scope");

    // A principal may only be recorded when an end is recorded. Expiring records
    // nobody, but naming a principal without an end is a half-written lift.
    let partial = sqlx::query("UPDATE mahfel.blocks SET lifted_by = $2 WHERE id = $1")
        .bind(fresh.id.as_uuid())
        .bind(manager.id.as_uuid())
        .execute(pool)
        .await;
    assert!(
        partial.is_err(),
        "a principal must not be recorded without an end"
    );

    // The mute columns move together, so a partial write cannot make a mute
    // permanent by omission.
    let participant = sqlx::query(
        "INSERT INTO mahfel.session_participants \
         (id, session_id, actor_id, status, joined_at, created_at, updated_at) \
         VALUES ($1, $2, $3, 'active', $4, $4, $4)",
    )
    .bind(Uuid::new_v4())
    .bind(session.id.as_uuid())
    .bind(member.id.as_uuid())
    .bind(base)
    .execute(pool)
    .await
    .expect("participant insert");

    let _ = participant;
    let half_muted = sqlx::query(
        "UPDATE mahfel.session_participants SET muted_at = $2 \
         WHERE session_id = $1",
    )
    .bind(session.id.as_uuid())
    .bind(base)
    .execute(pool)
    .await;
    assert!(
        half_muted.is_err(),
        "a mute must record when, how firmly, and by whom, or none of them"
    );

    Ok(())
}
