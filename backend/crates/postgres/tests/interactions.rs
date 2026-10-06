use mahfel_application::ports::{ParticipantStore, RoomStore, SessionStore, UserStore};
use mahfel_application::{
    interactions,
    ports::{ActivityStatus, InteractionCommand, InteractionError, InteractionStore, Principal},
};
use mahfel_domain::{ActivityDefinition, ActivityKind, ActorId, ActorKind};
use mahfel_domain::{
    DisplayName, Participant, Room, RoomMember, RoomRole, RoomTitle, RoomVisibility, Session, User,
};
use mahfel_postgres::PostgresInteractionStore;
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
async fn interactions_are_atomic_private_and_recoverable() -> Result<(), Box<dyn Error>> {
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

    let verification = verify_interactions(&pool).await;
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

fn principal(actor_id: ActorId) -> Principal {
    Principal {
        actor_id,
        kind: ActorKind::User,
        display_name: None,
        avatar_url: None,
        email: None,
        email_verified: false,
        username: None,
        roles: Vec::new(),
    }
}
fn definition(kind: ActivityKind) -> ActivityDefinition {
    ActivityDefinition {
        question: "Prime numbers?".into(),
        options: vec!["2".into(), "3".into(), "4".into()],
        kind,
        multiple: true,
        anonymous: true,
        correct_options: if kind == ActivityKind::Quiz {
            vec![0, 1]
        } else {
            vec![]
        },
        explanation: None,
    }
}
async fn verify_interactions(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    let now = OffsetDateTime::now_utc();
    let users = PostgresUserStore::new(pool.clone());
    let rooms = PostgresRoomStore::new(pool.clone());
    let sessions = PostgresSessionStore::new(pool.clone());
    let participants = PostgresParticipantStore::new(pool.clone());
    let store = PostgresInteractionStore::new(pool.clone());
    let mut actors = Vec::new();
    for name in ["Host", "Member", "Other", "Outsider"] {
        let user = User::create(
            DisplayName::new(name.into())?,
            mahfel_domain::EmailAddress::new(format!("{}@example.test", name.to_lowercase()))?,
            mahfel_domain::Username::new(name.to_lowercase())?,
            now,
        );
        users.insert(&user).await?;
        actors.push(user.id);
    }
    let host = principal(actors[0]);
    let member = principal(actors[1]);
    let other = principal(actors[2]);
    let outsider = principal(actors[3]);
    let room = Room::create(
        RoomTitle::new("Activities".into())?,
        host.actor_id,
        RoomVisibility::InviteOnly,
        now,
    );
    rooms
        .create_with_manager(
            &room,
            &RoomMember::new(room.id, host.actor_id, RoomRole::Manager, now),
        )
        .await?;
    for actor in [member.actor_id, other.actor_id] {
        rooms
            .add_member(&RoomMember::new(room.id, actor, RoomRole::Member, now))
            .await?;
    }
    let mut session = Session::schedule(room.id, None, now);
    session.start(now)?;
    sessions.insert(&session).await?;
    let member_participant = Participant::join(session.id, member.actor_id, now);
    let other_participant = Participant::join(session.id, other.actor_id, now);
    participants.insert(&member_participant).await?;
    participants.insert(&other_participant).await?;
    assert!(matches!(
        store.snapshot(session.id, &outsider).await,
        Err(InteractionError::NotFound)
    ));
    let poll = Uuid::new_v4();
    assert!(
        store
            .execute(
                session.id,
                &member,
                Uuid::new_v4(),
                InteractionCommand::SaveDraft {
                    id: poll,
                    definition: definition(ActivityKind::Poll)
                }
            )
            .await
            .is_err()
    );
    store
        .execute(
            session.id,
            &host,
            Uuid::new_v4(),
            InteractionCommand::SaveDraft {
                id: poll,
                definition: definition(ActivityKind::Poll),
            },
        )
        .await?;
    assert!(
        store
            .snapshot(session.id, &member)
            .await?
            .activities
            .is_empty()
    );
    store
        .execute(
            session.id,
            &host,
            Uuid::new_v4(),
            InteractionCommand::Launch { id: poll },
        )
        .await?;
    let request = Uuid::new_v4();
    let vote = InteractionCommand::Vote {
        id: poll,
        selected: vec![1, 0],
        expected_version: 0,
    };
    store
        .execute(session.id, &member, request, vote.clone())
        .await?;
    let version = store.version(session.id).await?;
    store
        .execute(session.id, &member, request, vote.clone())
        .await?;
    assert_eq!(store.version(session.id).await?, version);
    let first = store.snapshot(session.id, &member).await?;
    let first_result =
        interactions::results(&first.activities[0], member.actor_id).expect("voter results");
    assert_eq!(first_result.counts, vec![1, 1, 0]);
    assert_eq!(first_result.respondents, 1);
    assert!(interactions::results(&first.activities[0], other.actor_id).is_none());
    store
        .execute(
            session.id,
            &member,
            Uuid::new_v4(),
            InteractionCommand::Retract {
                id: poll,
                expected_version: 1,
            },
        )
        .await?;
    store.execute(session.id, &member, request, vote).await?;
    assert_eq!(
        store.snapshot(session.id, &member).await?.activities[0].votes[0].selected,
        Vec::<u16>::new()
    );
    assert!(
        store
            .execute(
                session.id,
                &member,
                Uuid::new_v4(),
                InteractionCommand::Vote {
                    id: poll,
                    selected: vec![2],
                    expected_version: 0
                }
            )
            .await
            .is_err()
    );
    let (left, right) = tokio::join!(
        store.execute(
            session.id,
            &member,
            Uuid::new_v4(),
            InteractionCommand::Vote {
                id: poll,
                selected: vec![0],
                expected_version: 2
            }
        ),
        store.execute(
            session.id,
            &member,
            Uuid::new_v4(),
            InteractionCommand::Vote {
                id: poll,
                selected: vec![2],
                expected_version: 2
            }
        )
    );
    assert!(
        left.is_ok() ^ right.is_ok(),
        "one concurrent vote replacement wins"
    );
    store
        .execute(
            session.id,
            &host,
            Uuid::new_v4(),
            InteractionCommand::Close { id: poll },
        )
        .await?;
    let quiz = Uuid::new_v4();
    store
        .execute(
            session.id,
            &host,
            Uuid::new_v4(),
            InteractionCommand::SaveDraft {
                id: quiz,
                definition: definition(ActivityKind::Quiz),
            },
        )
        .await?;
    store
        .execute(
            session.id,
            &host,
            Uuid::new_v4(),
            InteractionCommand::Launch { id: quiz },
        )
        .await?;
    store
        .execute(
            session.id,
            &member,
            Uuid::new_v4(),
            InteractionCommand::Vote {
                id: quiz,
                selected: vec![0],
                expected_version: 0,
            },
        )
        .await?;
    store
        .execute(
            session.id,
            &other,
            Uuid::new_v4(),
            InteractionCommand::Vote {
                id: quiz,
                selected: vec![0, 1],
                expected_version: 0,
            },
        )
        .await?;
    let current = store.snapshot(session.id, &host).await?;
    let q = current
        .activities
        .iter()
        .find(|a| a.id == quiz)
        .expect("quiz");
    assert!(interactions::results(q, host.actor_id).is_none());
    assert!(
        store
            .execute(
                session.id,
                &member,
                Uuid::new_v4(),
                InteractionCommand::Vote {
                    id: quiz,
                    selected: vec![0, 1],
                    expected_version: 1
                }
            )
            .await
            .is_err()
    );
    store
        .execute(
            session.id,
            &host,
            Uuid::new_v4(),
            InteractionCommand::Close { id: quiz },
        )
        .await?;
    let current = store.snapshot(session.id, &member).await?;
    let q = current
        .activities
        .iter()
        .find(|a| a.id == quiz)
        .expect("quiz");
    assert_eq!(
        interactions::results(q, member.actor_id)
            .expect("results")
            .correct,
        Some(false)
    );
    assert_eq!(
        interactions::results(q, other.actor_id)
            .expect("results")
            .correct,
        Some(true)
    );
    let raise_id = Uuid::new_v4();
    let raise = InteractionCommand::SetHand {
        raised: true,
        expected_version: 0,
    };
    store
        .execute(session.id, &member, raise_id, raise.clone())
        .await?;
    store
        .execute(session.id, &other, Uuid::new_v4(), raise.clone())
        .await?;
    let hands = store.raised_hands(session.id).await?;
    assert_eq!(hands.len(), 2);
    assert_eq!(hands[0].actor_id, member.actor_id);
    let first_order = hands[0].order;
    store
        .execute(
            session.id,
            &host,
            Uuid::new_v4(),
            InteractionCommand::LowerHand {
                participant_id: member_participant.id,
                expected_order: first_order,
            },
        )
        .await?;
    store
        .execute(session.id, &member, raise_id, raise.clone())
        .await?;
    assert_eq!(
        store.raised_hands(session.id).await?.len(),
        1,
        "retry cannot resurrect lowered hand"
    );
    store
        .execute(session.id, &member, Uuid::new_v4(), raise)
        .await?;
    store
        .execute(
            session.id,
            &host,
            Uuid::new_v4(),
            InteractionCommand::LowerHand {
                participant_id: member_participant.id,
                expected_order: first_order,
            },
        )
        .await?;
    let hands = store.raised_hands(session.id).await?;
    assert_eq!(hands.len(), 2);
    assert_eq!(hands[1].actor_id, member.actor_id);
    let mut departing = other_participant;
    departing.remove(now)?;
    participants.save(&departing).await?;
    assert_eq!(store.raised_hands(session.id).await?.len(), 1);
    assert!(
        store
            .execute(
                session.id,
                &other,
                Uuid::new_v4(),
                InteractionCommand::SetHand {
                    raised: true,
                    expected_version: 0
                }
            )
            .await
            .is_err()
    );
    let open = Uuid::new_v4();
    let guest = Participant::join(session.id, outsider.actor_id, now);
    participants.insert(&guest).await?;
    sqlx::query("INSERT INTO mahfel.admission_requests (id, session_id, room_id, actor_id, status, origin, requested_at, decided_at, decided_by) VALUES ($1, $2, $3, $4, 'admitted', 'invite_link', now(), now(), $5)")
        .bind(Uuid::new_v4()).bind(session.id.as_uuid()).bind(room.id.as_uuid()).bind(outsider.actor_id.as_uuid()).bind(host.actor_id.as_uuid()).execute(pool).await?;
    assert!(
        store.can_respond(session.id, &outsider).await?,
        "session admission permits a guest without room membership"
    );
    sqlx::query("DELETE FROM mahfel.room_memberships WHERE room_id = $1 AND actor_id = $2")
        .bind(room.id.as_uuid())
        .bind(member.actor_id.as_uuid())
        .execute(pool)
        .await?;
    assert!(matches!(
        store.snapshot(session.id, &member).await,
        Err(InteractionError::NotFound)
    ));
    assert!(matches!(
        store
            .execute(
                session.id,
                &member,
                Uuid::new_v4(),
                InteractionCommand::SetHand {
                    raised: true,
                    expected_version: 0
                }
            )
            .await,
        Err(InteractionError::NotFound)
    ));
    rooms
        .add_member(&RoomMember::new(
            room.id,
            member.actor_id,
            RoomRole::Member,
            now,
        ))
        .await?;
    store
        .execute(
            session.id,
            &host,
            Uuid::new_v4(),
            InteractionCommand::SaveDraft {
                id: open,
                definition: definition(ActivityKind::Poll),
            },
        )
        .await?;
    store
        .execute(
            session.id,
            &host,
            Uuid::new_v4(),
            InteractionCommand::Launch { id: open },
        )
        .await?;
    session.end(now)?;
    sessions.save(&session).await?;
    assert!(store.raised_hands(session.id).await?.is_empty());
    let current = store.snapshot(session.id, &member).await?;
    assert!(!current.live);
    assert_eq!(
        current
            .activities
            .iter()
            .find(|a| a.id == open)
            .expect("poll")
            .status,
        ActivityStatus::Closed
    );
    assert!(
        store
            .execute(
                session.id,
                &member,
                Uuid::new_v4(),
                InteractionCommand::Vote {
                    id: open,
                    selected: vec![0],
                    expected_version: 0
                }
            )
            .await
            .is_err()
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
