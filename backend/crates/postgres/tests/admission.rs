use mahfel_application::ports::{AdmissionStore, RoomStore, SessionStore, StoreError, UserStore};
use mahfel_domain::{
    ActorId, AdmissionOrigin, AdmissionPolicy, AdmissionRequest, AdmissionRequestStatus,
    AdmissionTarget, DisplayName, Room, RoomMember, RoomRole, RoomTitle, RoomVisibility, Session,
    SessionVisibility, User,
};
use mahfel_postgres::{
    ConnectionSettings, PoolSettings, PostgresAdmissionStore, PostgresRoomStore,
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
async fn admission_requests_are_unique_and_decidable_once() -> Result<(), Box<dyn Error>> {
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
/// the server.
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

    let verification = verify_admission(&pool).await;
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

async fn verify_admission(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    let base = OffsetDateTime::now_utc()
        .replace_nanosecond(0)
        .expect("zero nanoseconds is valid");

    let users = PostgresUserStore::new(pool.clone());
    let rooms = PostgresRoomStore::new(pool.clone());
    let sessions = PostgresSessionStore::new(pool.clone());
    let admissions = PostgresAdmissionStore::new(pool.clone());

    let manager = User::create(DisplayName::new("Ada Lovelace".to_owned())?, base);
    users.insert(&manager).await?;
    let member = User::create(DisplayName::new("Grace Hopper".to_owned())?, base);
    users.insert(&member).await?;

    // The room policy and the session overrides must survive a round trip;
    // a mapper that omits a column would silently return the default.
    let mut room = Room::create(
        RoomTitle::new("Analytical Engines".to_owned())?,
        manager.id,
        RoomVisibility::Public,
        base,
    );
    room.set_admission(AdmissionPolicy::Lobby, base)?;
    rooms
        .create_with_manager(
            &room,
            &RoomMember::new(room.id, manager.id, RoomRole::Manager, base),
        )
        .await?;
    let stored_room = rooms.get(room.id).await?.expect("stored room");
    assert_eq!(stored_room.admission, AdmissionPolicy::Lobby);

    let mut session = Session::schedule(room.id, None, base);
    session.set_visibility(Some(SessionVisibility::InviteOnly), &room, base)?;
    session.set_admission(Some(AdmissionPolicy::Lobby), &room, base)?;
    sessions.insert(&session).await?;

    let stored_session = sessions.get(session.id).await?.expect("stored session");
    assert_eq!(
        stored_session.visibility,
        Some(SessionVisibility::InviteOnly),
        "a session override must round-trip, not fall back to the default"
    );
    assert_eq!(stored_session.admission, Some(AdmissionPolicy::Lobby));

    // An inherited session writes NULL, which must read back as None.
    let inherited = Session::schedule(room.id, None, base);
    sessions.insert(&inherited).await?;
    let stored_inherited = sessions.get(inherited.id).await?.expect("stored");
    assert!(stored_inherited.visibility.is_none());
    assert!(stored_inherited.admission.is_none());
    assert_eq!(
        stored_inherited.effective_admission(&stored_room),
        AdmissionPolicy::Lobby
    );

    // One pending request per (session, actor): a duplicate join cannot queue twice.
    let waiting = AdmissionRequest::new(
        AdmissionTarget::Session(session.id),
        member.id,
        AdmissionOrigin::Join,
        base,
    );
    admissions.insert(&waiting).await?;
    let duplicate = AdmissionRequest::new(
        AdmissionTarget::Session(session.id),
        member.id,
        AdmissionOrigin::InviteLink,
        base,
    );
    let conflict = admissions.insert(&duplicate).await;
    assert!(
        matches!(conflict, Err(StoreError::Conflict(_))),
        "a second pending request must conflict, got {conflict:?}"
    );

    assert_eq!(
        admissions
            .count_pending(AdmissionTarget::Session(session.id))
            .await?,
        1
    );
    let queue = admissions.list_pending(session.id, 10).await?;
    assert_eq!(queue.len(), 1);
    assert_eq!(queue[0].origin, AdmissionOrigin::Join);
    assert_eq!(
        admissions
            .get_pending(AdmissionTarget::Session(session.id), member.id)
            .await?
            .map(|r| r.id),
        Some(waiting.id)
    );

    // A decision applies once; a repeat reports no change so a concurrent
    // decision cannot both succeed.
    let decided_at = base + time::Duration::seconds(30);
    assert!(
        admissions
            .decide_if_pending(
                waiting.id,
                AdmissionRequestStatus::Admitted,
                Some(manager.id),
                decided_at
            )
            .await?
    );
    assert!(
        !admissions
            .decide_if_pending(
                waiting.id,
                AdmissionRequestStatus::Rejected,
                Some(manager.id),
                decided_at
            )
            .await?,
        "a decided request must not be decidable again"
    );
    let decided = admissions.get(waiting.id).await?.expect("retained");
    assert_eq!(decided.status, AdmissionRequestStatus::Admitted);
    assert_eq!(decided.decided_by, Some(manager.id));
    assert_eq!(
        admissions
            .count_pending(AdmissionTarget::Session(session.id))
            .await?,
        0
    );

    // The end-of-session sweep retires anything still pending, naming no
    // deciding principal because none made the call.
    let other = AdmissionRequest::new(
        AdmissionTarget::Session(session.id),
        ActorId::new(),
        AdmissionOrigin::InviteLink,
        base,
    );
    // An arbitrary actor id has no row, so this must fail on the foreign key.
    let dangling = admissions.insert(&other).await;
    assert!(
        matches!(dangling, Err(StoreError::MissingReference(_))),
        "a request must reference a real actor, got {dangling:?}"
    );

    let second_waiting = AdmissionRequest::new(
        AdmissionTarget::Session(session.id),
        manager.id,
        AdmissionOrigin::InviteLink,
        base,
    );
    admissions.insert(&second_waiting).await?;
    let swept = admissions
        .expire_pending_for_session(session.id, base + time::Duration::seconds(90))
        .await?;
    assert_eq!(swept, 1, "only the pending request is retired");
    assert_eq!(
        admissions
            .expire_pending_for_session(session.id, base + time::Duration::seconds(120))
            .await?,
        0,
        "the sweep is idempotent"
    );
    let expired = admissions.get(second_waiting.id).await?.expect("retained");
    assert_eq!(expired.status, AdmissionRequestStatus::Expired);
    assert!(
        expired.decided_by.is_none(),
        "an expiry names no deciding principal"
    );
    assert_eq!(expired.decided_at, Some(base + time::Duration::seconds(90)));

    // The schema refuses a pending request that carries a decision.
    let malformed = sqlx::query(
        "INSERT INTO mahfel.admission_requests \
         (id, session_id, actor_id, status, origin, requested_at, decided_at, decided_by) \
         VALUES ($1, $2, $3, 'pending', 'join', $4, $5, $6)",
    )
    .bind(Uuid::new_v4())
    .bind(session.id.as_uuid())
    .bind(manager.id.as_uuid())
    .bind(base)
    .bind(decided_at)
    .bind(manager.id.as_uuid())
    .execute(pool)
    .await;
    assert!(
        malformed.is_err(),
        "a pending request must not record a decision"
    );

    // A room-scoped request exists with no session, which is the whole point
    // of the 0010 migration.
    let room_request = AdmissionRequest::new(
        AdmissionTarget::Room(room.id),
        member.id,
        AdmissionOrigin::InviteLink,
        base,
    );
    admissions.insert(&room_request).await?;

    // The room is always present, because authorization resolves against it.
    let no_room = sqlx::query(
        "INSERT INTO mahfel.admission_requests \
         (id, session_id, room_id, actor_id, status, origin, requested_at) \
         VALUES ($1, NULL, NULL, $2, 'pending', 'join', $3)",
    )
    .bind(Uuid::new_v4())
    .bind(manager.id.as_uuid())
    .bind(base)
    .execute(pool)
    .await;
    assert!(no_room.is_err(), "every request must name a room");

    // A room request is listed on the room queue, not a session queue.
    let room_queue = admissions.list_pending_for_room(room.id, 10).await?;
    assert_eq!(room_queue.len(), 1);
    assert_eq!(room_queue[0].target, AdmissionTarget::Room(room.id));
    assert!(
        admissions
            .list_pending(session.id, 10)
            .await?
            .iter()
            .all(|request| request.target != AdmissionTarget::Room(room.id)),
        "a room request must not appear on a session queue"
    );
    assert_eq!(
        admissions
            .count_pending(AdmissionTarget::Room(room.id))
            .await?,
        1
    );

    // A room request is never retired by a session ending.
    let swept_room_scope = admissions
        .expire_pending_for_session(session.id, base + time::Duration::seconds(300))
        .await?;
    assert_eq!(swept_room_scope, 0, "room requests outlive sessions");
    assert_eq!(
        admissions
            .count_pending(AdmissionTarget::Room(room.id))
            .await?,
        1
    );

    let bad_policy = sqlx::query("UPDATE mahfel.rooms SET admission = 'waiting' WHERE id = $1")
        .bind(room.id.as_uuid())
        .execute(pool)
        .await;
    assert!(
        bad_policy.is_err(),
        "the policy CHECK must reject an unknown value"
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
