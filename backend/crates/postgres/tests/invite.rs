use mahfel_application::ports::{InviteStore, RoomStore, SessionStore, StoreError, UserStore};
use mahfel_domain::{
    ActorId, DEFAULT_EXPIRY_DAYS, DisplayName, InviteCode, InviteLink, InviteLinkStatus,
    InviteTarget, Room, RoomMember, RoomRole, RoomTitle, RoomVisibility, Session, User,
};
use mahfel_postgres::{
    ConnectionSettings, PoolSettings, PostgresInviteStore, PostgresRoomStore, PostgresSessionStore,
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
async fn invite_links_are_unique_and_revocable() -> Result<(), Box<dyn Error>> {
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

    let verification = verify_invites(&pool).await;
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

async fn verify_invites(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    let now = OffsetDateTime::now_utc()
        .replace_nanosecond(0)
        .expect("zero nanoseconds is valid");
    let users = PostgresUserStore::new(pool.clone());
    let rooms = PostgresRoomStore::new(pool.clone());
    let sessions = PostgresSessionStore::new(pool.clone());
    let invites = PostgresInviteStore::new(pool.clone());

    let owner = User::create(
        DisplayName::new("Ada Lovelace".to_owned())?,
        mahfel_domain::EmailAddress::new("test@example.test".to_owned())?,
        mahfel_domain::Username::new("test-user".to_owned())?,
        now,
    );
    users.insert(&owner).await?;

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
    let session = Session::schedule(room.id, None, now);
    sessions.insert(&session).await?;

    // A room link and a session link share one namespace.
    let room_link = link_for(InviteTarget::Room(room.id), owner.id, now)?;
    invites.insert(&room_link).await?;
    let session_link = link_for(InviteTarget::Session(session.id), owner.id, now)?;
    invites.insert(&session_link).await?;

    // The code is the lookup key and is canonical on the way in and out.
    let found = invites
        .get_by_code(room_link.code)
        .await?
        .expect("stored by code");
    assert_eq!(found.id, room_link.id);
    assert_eq!(found.target, InviteTarget::Room(room.id));
    assert_eq!(found.status, InviteLinkStatus::Active);
    assert!(found.revoked_at.is_none());

    // Redemption normalizes before lookup, so a mixed-case paste resolves.
    let typed = InviteCode::parse(&room_link.code.format().to_uppercase())?;
    assert_eq!(
        invites.get_by_code(typed).await?.expect("normalized").id,
        room_link.id
    );

    // A second row cannot claim the same code: uniqueness is the database's job.
    let duplicate = InviteLink {
        code: room_link.code,
        ..link_for(InviteTarget::Room(room.id), owner.id, now)?
    };
    let conflict = invites.insert(&duplicate).await;
    assert!(
        matches!(conflict, Err(StoreError::Conflict(_))),
        "a duplicate code must conflict, got {conflict:?}"
    );

    // Listing a target returns newest first and does not leak the other target.
    let for_room = invites
        .list_for_target(mahfel_domain::InviteTargetKind::Room, room.id.as_uuid(), 10)
        .await?;
    assert_eq!(for_room.len(), 1);
    assert_eq!(for_room[0].id, room_link.id);

    // Revocation is terminal and reports a real change exactly once.
    assert!(invites.revoke_if_active(session_link.id, now).await?);
    let refused = invites.revoke_if_active(session_link.id, now).await?;
    assert!(!refused, "a repeat revoke must not report a change");
    let revoked = invites
        .get(session_link.id)
        .await?
        .expect("retained after revoke");
    assert_eq!(revoked.status, InviteLinkStatus::Revoked);
    assert_eq!(revoked.revoked_at, Some(now));
    assert!(!revoked.is_redeemable_at(now));
    // The row is retained, so history survives revocation.
    assert_eq!(
        invites
            .list_for_target(
                mahfel_domain::InviteTargetKind::Session,
                session.id.as_uuid(),
                10
            )
            .await?
            .len(),
        1
    );

    // Rotation revokes the previous code and issues a replacement atomically.
    let replacement = InviteLink {
        id: mahfel_domain::InviteLinkId::new(),
        code: InviteCode::generate(),
        target: InviteTarget::Room(room.id),
        status: InviteLinkStatus::Active,
        created_by: owner.id,
        expires_at: room_link.expires_at,
        created_at: now,
        revoked_at: None,
    };
    let rotated_at = now + time::Duration::hours(1);
    invites
        .rotate(room_link.id, rotated_at, &replacement)
        .await?;

    let previous = invites.get(room_link.id).await?.expect("retained");
    assert_eq!(previous.status, InviteLinkStatus::Revoked);
    assert_eq!(previous.revoked_at, Some(rotated_at));
    assert!(
        !previous.is_redeemable_at(rotated_at),
        "the retired code must stop working"
    );
    assert!(
        invites
            .get_by_code(replacement.code)
            .await?
            .expect("replacement")
            .is_redeemable_at(rotated_at),
        "the replacement must be live"
    );
    assert_eq!(
        invites
            .list_for_target(mahfel_domain::InviteTargetKind::Room, room.id.as_uuid(), 10)
            .await?
            .len(),
        2,
        "both generations are retained"
    );

    // A rotation of an already-revoked link must fail closed.
    let again = invites
        .rotate(session_link.id, rotated_at, &replacement)
        .await;
    assert!(
        matches!(again, Err(StoreError::Conflict(_))),
        "rotating a revoked link must conflict, got {again:?}"
    );

    // The schema refuses a malformed code and a mismatched revocation state, so
    // the domain rules are also enforced at the database.
    let bad_code = sqlx::query(
        "INSERT INTO mahfel.invite_links \
         (id, code, target_kind, target_id, status, expires_at, created_by, created_at, revoked_at) \
         VALUES ($1, 'NOT-A-CODE', 'room', $2, 'active', $3, $4, $5, NULL)",
    )
    .bind(Uuid::new_v4())
    .bind(room.id.as_uuid())
    .bind(now + time::Duration::days(1))
    .bind(owner.id.as_uuid())
    .bind(now)
    .execute(pool)
    .await;
    assert!(bad_code.is_err(), "the code CHECK must reject a bad shape");

    let mismatched = sqlx::query(
        "INSERT INTO mahfel.invite_links \
         (id, code, target_kind, target_id, status, expires_at, created_by, created_at, revoked_at) \
         VALUES ($1, $2, 'room', $3, 'revoked', $4, $5, $6, NULL)",
    )
    .bind(Uuid::new_v4())
    .bind(InviteCode::generate().as_str())
    .bind(room.id.as_uuid())
    .bind(now + time::Duration::days(1))
    .bind(owner.id.as_uuid())
    .bind(now)
    .execute(pool)
    .await;
    assert!(
        mismatched.is_err(),
        "revoked without a timestamp must be rejected"
    );

    Ok(())
}

fn link_for(
    target: InviteTarget,
    created_by: ActorId,
    now: OffsetDateTime,
) -> Result<InviteLink, Box<dyn Error>> {
    Ok(InviteLink::issue(
        InviteCode::generate(),
        target,
        created_by,
        DEFAULT_EXPIRY_DAYS,
        now,
    )?)
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
