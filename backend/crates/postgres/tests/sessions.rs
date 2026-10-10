use mahfel_application::ports::{
    MediaPlacement, MediaPlacementStore, RoomStore, SessionDetailsFields, SessionStartClaim,
    SessionStore, StoreError, UserStore,
};
use mahfel_domain::{
    DisplayName, NodeId, Room, RoomId, RoomMember, RoomRole, RoomTitle, RoomVisibility, Session,
    SessionDurationMinutes, SessionStatus, SessionTitle, User,
};
use mahfel_postgres::{
    ConnectionSettings, PoolSettings, PostgresMediaPlacementStore, PostgresRoomStore,
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

    let user = User::create(
        DisplayName::new("Ada Lovelace".to_owned())?,
        mahfel_domain::EmailAddress::new("test@example.test".to_owned())?,
        mahfel_domain::Username::new("test-user".to_owned())?,
        now,
    );
    users.insert(&user).await?;
    let seed_room = Room::create(
        RoomTitle::new("Seeded classroom".to_owned())?,
        user.id,
        RoomVisibility::InviteOnly,
        now,
    );
    let seed_manager = RoomMember::new(seed_room.id, user.id, RoomRole::Manager, now);
    let mut seed_session = Session::schedule(seed_room.id, None, now);
    seed_session.set_details(
        Some(SessionTitle::new("Seeded lesson".to_owned())?),
        None,
        None,
        now,
    );
    for _ in 0..2 {
        rooms
            .seed_development_room(
                &seed_room,
                std::slice::from_ref(&seed_manager),
                std::slice::from_ref(&seed_session),
            )
            .await?;
    }
    assert_eq!(rooms.get(seed_room.id).await?, Some(seed_room.clone()));
    assert_eq!(rooms.list_members(seed_room.id).await?, vec![seed_manager]);
    assert_eq!(
        sessions.list_for_room(seed_room.id).await?,
        vec![seed_session.clone()]
    );
    let standalone_session = Session::schedule(seed_room.id, None, now + Duration::from_secs(1));
    sessions
        .seed_development_session(&standalone_session)
        .await?;
    sessions
        .seed_development_session(&standalone_session)
        .await?;
    assert_eq!(
        sessions.list_for_room(seed_room.id).await?,
        vec![standalone_session, seed_session.clone()]
    );

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
    let mut first = Session::schedule(room.id, None, now);
    first.set_details(
        Some(SessionTitle::new("Lecture".to_owned())?),
        None,
        Some(SessionDurationMinutes::new(90)?),
        now,
    );
    sessions.insert(&first).await?;
    let second = Session::schedule(room.id, None, now + Duration::from_secs(1));
    sessions.insert(&second).await?;

    // Listing is scoped to the room and returns newest first.
    let listed = sessions.list_for_room(room.id).await?;
    assert_eq!(listed, vec![second.clone(), first.clone()]);
    assert_eq!(
        sessions.list_for_room_page(room.id, None, 1, 0).await?,
        vec![second.clone()]
    );
    assert_eq!(
        sessions.list_for_room_page(room.id, None, 1, 1).await?,
        vec![first.clone()]
    );
    assert!(
        sessions
            .list_for_room_page(room.id, None, 1, 2)
            .await?
            .is_empty()
    );
    assert!(
        sessions
            .list_for_room_page(room.id, Some(SessionStatus::Live), 20, 0)
            .await?
            .is_empty()
    );
    assert!(
        sessions.list_for_room(RoomId::new()).await?.is_empty(),
        "another room has no sessions"
    );

    let stale_details = first.clone();
    let invalid_duration =
        sqlx::query("UPDATE mahfel.sessions SET scheduled_duration_minutes = 10081 WHERE id = $1")
            .bind(first.id.as_uuid())
            .execute(pool)
            .await;
    assert!(
        invalid_duration.is_err(),
        "database must reject overlong duration"
    );
    let invalid_title = sqlx::query("UPDATE mahfel.sessions SET title = '' WHERE id = $1")
        .bind(first.id.as_uuid())
        .execute(pool)
        .await;
    assert!(invalid_title.is_err(), "database must reject empty title");

    // Only one session may be live at a time; the losing write conflicts.
    first.start(now + Duration::from_secs(1))?;
    sessions.save(&first).await?;
    assert_eq!(
        sessions
            .list_for_room_page(room.id, Some(SessionStatus::Live), 1, 0)
            .await?,
        vec![first.clone()]
    );
    let mut stale_details = stale_details;
    stale_details.set_details(None, None, None, now + Duration::from_secs(2));
    let after_details = sessions
        .save_details(
            &stale_details,
            SessionDetailsFields {
                auto_record: false,
                title: true,
                scheduled_start_at: true,
                scheduled_duration_minutes: true,
            },
        )
        .await?;
    assert_eq!(after_details.status, SessionStatus::Live);
    assert!(after_details.title.is_none());
    assert!(after_details.scheduled_duration_minutes.is_none());

    // Two edits based on the same old row must preserve independently
    // supplied fields even when they finish in sequence.
    let mut title_edit = after_details.clone();
    let mut duration_edit = after_details.clone();
    title_edit.title = Some(SessionTitle::new("Revised".to_owned())?);
    duration_edit.scheduled_duration_minutes = Some(SessionDurationMinutes::new(45)?);
    sessions
        .save_details(
            &title_edit,
            SessionDetailsFields {
                auto_record: false,
                title: true,
                ..SessionDetailsFields::default()
            },
        )
        .await?;
    let combined = sessions
        .save_details(
            &duration_edit,
            SessionDetailsFields {
                scheduled_duration_minutes: true,
                ..SessionDetailsFields::default()
            },
        )
        .await?;
    assert_eq!(
        combined.title.as_ref().map(SessionTitle::as_str),
        Some("Revised")
    );
    assert_eq!(
        combined.scheduled_duration_minutes.map(|value| value.get()),
        Some(45)
    );

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

    // The unique durable decision wins even if a later lease returns another
    // node. Operators can query the recorded capacity and reason directly.
    let placements = PostgresMediaPlacementStore::new(pool.clone());
    let chosen = MediaPlacement {
        session_id: first.id,
        room_id: room.id,
        node_id: NodeId::from_deployment("media-a").expect("valid node id"),
        max_rooms: 12,
        active_rooms: 3,
        reason: "weighted_available_capacity".to_owned(),
        decided_at: now,
    };
    assert_eq!(
        placements.insert_or_get(&chosen).await?.node_id,
        chosen.node_id
    );
    let competing = MediaPlacement {
        node_id: NodeId::from_deployment("media-b").expect("valid node id"),
        ..chosen.clone()
    };
    assert_eq!(
        placements.insert_or_get(&competing).await?.node_id,
        chosen.node_id
    );
    let diagnostic: (String, i32, i64, String) = sqlx::query_as(
        "SELECT node_id, max_rooms, active_rooms, reason FROM mahfel.media_placements WHERE session_id = $1",
    )
    .bind(first.id.as_uuid())
    .fetch_one(pool)
    .await?;
    assert_eq!(diagnostic.0, "media-a");
    assert_eq!(diagnostic.1, 12);
    assert_eq!(diagnostic.2, 3);
    assert_eq!(diagnostic.3, "weighted_available_capacity");

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

    let gate_room = Room::create(
        RoomTitle::new("Recorded start gate".to_owned())?,
        user.id,
        RoomVisibility::InviteOnly,
        now,
    );
    rooms
        .create_with_manager(
            &gate_room,
            &RoomMember::new(gate_room.id, user.id, RoomRole::Manager, now),
        )
        .await?;
    let gated = Session::schedule(gate_room.id, None, now);
    sessions.insert(&gated).await?;
    let SessionStartClaim::RecordingLease(expired) = sessions.claim_start(gated.id).await? else {
        panic!("auto recording should require a preflight lease");
    };
    assert!(matches!(
        sessions.claim_start(gated.id).await,
        Err(StoreError::Conflict(_))
    ));
    assert_eq!(
        sessions.get(gated.id).await?.expect("session").status,
        SessionStatus::Scheduled
    );
    sqlx::query("UPDATE mahfel.sessions SET recording_start_expires_at = clock_timestamp() - INTERVAL '1 second' WHERE id = $1")
        .bind(gated.id.as_uuid()).execute(pool).await?;
    let SessionStartClaim::RecordingLease(current) = sessions.claim_start(gated.id).await? else {
        panic!("expired preflight lease should be reclaimable");
    };
    assert_ne!(current, expired);
    assert!(matches!(
        sessions.commit_recorded_start(gated.id, expired).await,
        Err(StoreError::Conflict(_))
    ));
    sessions.release_recorded_start(gated.id, current).await?;
    assert_eq!(
        sessions.get(gated.id).await?.expect("session").status,
        SessionStatus::Scheduled
    );

    let SessionStartClaim::RecordingLease(token) = sessions.claim_start(gated.id).await? else {
        panic!("released lease should be reclaimable");
    };
    let started = sessions.commit_recorded_start(gated.id, token).await?;
    assert_eq!(started.status, SessionStatus::Live);
    let recording_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM mahfel.recordings WHERE session_id = $1")
            .bind(gated.id.as_uuid())
            .fetch_one(pool)
            .await?;
    assert_eq!(recording_count, 1);

    let unrecorded_room = Room::create(
        RoomTitle::new("Unrecorded start".to_owned())?,
        user.id,
        RoomVisibility::InviteOnly,
        now,
    );
    rooms
        .create_with_manager(
            &unrecorded_room,
            &RoomMember::new(unrecorded_room.id, user.id, RoomRole::Manager, now),
        )
        .await?;
    let mut unrecorded = Session::schedule(unrecorded_room.id, None, now);
    unrecorded.auto_record = false;
    sessions.insert(&unrecorded).await?;
    let SessionStartClaim::Started(started) = sessions.claim_start(unrecorded.id).await? else {
        panic!("unrecorded session should start without preflight");
    };
    assert_eq!(started.status, SessionStatus::Live);
    let recording_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM mahfel.recordings WHERE session_id = $1")
            .bind(unrecorded.id.as_uuid())
            .fetch_one(pool)
            .await?;
    assert_eq!(recording_count, 0);

    let preference_room = Room::create(
        RoomTitle::new("Recording preference race".to_owned())?,
        user.id,
        RoomVisibility::InviteOnly,
        now,
    );
    rooms
        .create_with_manager(
            &preference_room,
            &RoomMember::new(preference_room.id, user.id, RoomRole::Manager, now),
        )
        .await?;
    let mut preference = Session::schedule(preference_room.id, None, now);
    sessions.insert(&preference).await?;
    let SessionStartClaim::RecordingLease(token) = sessions.claim_start(preference.id).await?
    else {
        panic!("auto recording should acquire a lease");
    };
    preference.auto_record = false;
    sessions
        .save_details(
            &preference,
            SessionDetailsFields {
                auto_record: true,
                ..Default::default()
            },
        )
        .await?;
    assert!(matches!(
        sessions.commit_recorded_start(preference.id, token).await,
        Err(StoreError::Conflict(_))
    ));
    assert_eq!(
        sessions.get(preference.id).await?.expect("session").status,
        SessionStatus::Scheduled
    );
    let SessionStartClaim::Started(started) = sessions.claim_start(preference.id).await? else {
        panic!("disabled auto recording should bypass preflight");
    };
    assert_eq!(started.status, SessionStatus::Live);

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
