use mahfel_application::{
    ports::{Principal, RecordingStore, RoomStore, SessionDetailsFields, SessionStore, UserStore},
    recordings::{RecordingOperationError, control_recording},
};
use mahfel_domain::{
    ActorKind, DisplayName, EmailAddress, RecordingAction, RecordingStatus, Room, RoomMember,
    RoomRole, RoomTitle, RoomVisibility, Session, User, Username,
};
use mahfel_postgres::{PostgresStores, migrate};
use sqlx::PgPool;
use std::error::Error;
use testcontainers::{
    GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};
use time::OffsetDateTime;
use uuid::Uuid;

fn principal(user: &User, roles: &[&str]) -> Principal {
    Principal {
        actor_id: user.id,
        kind: ActorKind::User,
        display_name: None,
        avatar_url: None,
        email: None,
        email_verified: false,
        username: None,
        roles: roles.iter().map(|role| (*role).to_owned()).collect(),
    }
}

#[tokio::test]
async fn recording_commands_are_authorized_serialized_and_audited() -> Result<(), Box<dyn Error>> {
    if std::env::var("MAHFEL_TESTCONTAINERS").as_deref() != Ok("1") {
        eprintln!("MAHFEL_TESTCONTAINERS=1 required for recording PostgreSQL proof");
        return Ok(());
    }
    let container = GenericImage::new("postgres", "16-bookworm")
        .with_exposed_port(5432.tcp())
        .with_wait_for(WaitFor::message_on_stdout(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_PASSWORD", "mahfel-test")
        .with_env_var("POSTGRES_DB", "mahfel")
        .start()
        .await?;
    let port = container.get_host_port_ipv4(5432).await?;
    let database_url = format!("postgres://postgres:mahfel-test@127.0.0.1:{port}/mahfel");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let pool = loop {
        match PgPool::connect(&database_url).await {
            Ok(pool) => break pool,
            Err(_) if std::time::Instant::now() < deadline => {
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            }
            Err(error) => return Err(error.into()),
        }
    };
    migrate(&pool).await?;
    let stores = PostgresStores::new(pool.clone());
    let now = OffsetDateTime::now_utc();
    let manager = User::create(
        DisplayName::new("Manager".into())?,
        EmailAddress::new("manager@example.test".into())?,
        Username::new("manager".into())?,
        now,
    );
    let member = User::create(
        DisplayName::new("Member".into())?,
        EmailAddress::new("member@example.test".into())?,
        Username::new("member".into())?,
        now,
    );
    let outsider = User::create(
        DisplayName::new("Outsider".into())?,
        EmailAddress::new("outsider@example.test".into())?,
        Username::new("outsider".into())?,
        now,
    );
    for user in [&manager, &member, &outsider] {
        stores.users.insert(user).await?;
    }
    let room = Room::create(
        RoomTitle::new("Recording test".into())?,
        manager.id,
        RoomVisibility::InviteOnly,
        now,
    );
    stores
        .rooms
        .create_with_manager(
            &room,
            &RoomMember::new(room.id, manager.id, RoomRole::Manager, now),
        )
        .await?;
    stores
        .rooms
        .add_member(&RoomMember::new(room.id, member.id, RoomRole::Member, now))
        .await?;
    let mut session = Session::schedule(room.id, None, now);
    session.set_auto_record(false)?;
    stores.sessions.insert(&session).await?;
    session.start(now)?;
    stores.sessions.save(&session).await?;
    assert!(stores.recordings.list(session.id).await?.is_empty());
    let manager_principal = principal(&manager, &[]);
    let denied = control_recording(
        &stores.recordings,
        &stores.rooms,
        &stores.sessions,
        session.id,
        &principal(&member, &[]),
        Uuid::new_v4(),
        RecordingAction::Start,
    )
    .await;
    assert!(matches!(denied, Err(RecordingOperationError::Forbidden)));
    let denied = control_recording(
        &stores.recordings,
        &stores.rooms,
        &stores.sessions,
        session.id,
        &principal(&outsider, &["moderator"]),
        Uuid::new_v4(),
        RecordingAction::Start,
    )
    .await;
    assert!(matches!(denied, Err(RecordingOperationError::NotFound)));
    let document = Uuid::new_v4();
    sqlx::query("INSERT INTO mahfel.collaboration_documents (id, session_id, kind, schema_version) VALUES ($1,$2,'notes',1)")
        .bind(document).bind(session.id.as_uuid()).execute(&pool).await?;
    sqlx::query("INSERT INTO mahfel.collaboration_events (session_id, event_sequence, id, actor_id, document_id, kind, update_bytes) VALUES ($1,1,$2,$3,$4,'document_update',$5)")
        .bind(session.id.as_uuid()).bind(Uuid::new_v4()).bind(manager.id.as_uuid()).bind(document).bind(vec![0_u8,0]).execute(&pool).await?;
    let channel = Uuid::new_v4();
    sqlx::query("INSERT INTO mahfel.chat_channels (id, room_id, created_at) VALUES ($1,$2,clock_timestamp())")
        .bind(channel).bind(room.id.as_uuid()).execute(&pool).await?;
    let command_id = Uuid::new_v4();
    let start = control_recording(
        &stores.recordings,
        &stores.rooms,
        &stores.sessions,
        session.id,
        &manager_principal,
        command_id,
        RecordingAction::Start,
    )
    .await?;
    let duplicate = stores
        .recordings
        .command(session.id, manager.id, command_id, RecordingAction::Start)
        .await?;
    assert_eq!(start.id, duplicate.id);
    assert_eq!(start.revision, duplicate.revision);
    assert!(
        stores
            .recordings
            .command(session.id, manager.id, command_id, RecordingAction::Stop)
            .await
            .is_err()
    );
    let boot = Uuid::new_v4();
    let observation = mahfel_postgres::CaptureObservation {
        boot,
        origin: 1000,
        capturing: true,
        sealed: false,
        warning: None,
    };
    let initial_intent = stores
        .recordings
        .capture_intents(Some(session.id))
        .await?
        .remove(0);
    stores
        .recordings
        .capture_unavailable(&initial_intent)
        .await?;
    assert!(
        stores
            .recordings
            .capture_report(&initial_intent, &observation)
            .await?
    );
    let message = Uuid::new_v4();
    sqlx::query("INSERT INTO mahfel.chat_messages (id, channel_id, author_actor_id, body, created_at, session_id) VALUES ($1,$2,$3,'recorded',clock_timestamp(),$4)")
        .bind(message).bind(channel).bind(manager.id.as_uuid()).bind(session.id.as_uuid()).execute(&pool).await?;
    sqlx::query("INSERT INTO mahfel.collaboration_events (session_id, event_sequence, id, actor_id, document_id, kind, update_bytes) VALUES ($1,2,$2,$3,$4,'document_update',$5)")
        .bind(session.id.as_uuid()).bind(Uuid::new_v4()).bind(manager.id.as_uuid()).bind(document).bind(vec![0_u8,0]).execute(&pool).await?;
    let asset = Uuid::new_v4();
    sqlx::query("INSERT INTO mahfel.files (id, object_key, owner_actor_id, session_id, status, file_name, declared_mime, verified_mime, size_bytes, checksum_sha_256, created_at, updated_at) VALUES ($1,$2,$3,$4,'ready','fixture.pdf','application/pdf','application/pdf',8,'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=',clock_timestamp(),clock_timestamp())")
        .bind(asset).bind(format!("files/{asset}")).bind(manager.id.as_uuid()).bind(session.id.as_uuid()).execute(&pool).await?;
    sqlx::query(
        "INSERT INTO mahfel.collaboration_document_assets (document_id,file_id) VALUES ($1,$2)",
    )
    .bind(document)
    .bind(asset)
    .execute(&pool)
    .await?;
    use mahfel_application::ports::FileStore;
    assert!(
        stores
            .files
            .recording_pinned(mahfel_domain::FileId::from_uuid(asset))
            .await?
    );
    let deletion = sqlx::query("DELETE FROM mahfel.files WHERE id = $1")
        .bind(asset)
        .execute(&pool)
        .await
        .expect_err("pending archive protects metadata");
    assert!(
        deletion
            .to_string()
            .contains("pinned by a pending recording archive")
    );
    let pause = control_recording(
        &stores.recordings,
        &stores.rooms,
        &stores.sessions,
        session.id,
        &principal(&member, &["moderator"]),
        Uuid::new_v4(),
        RecordingAction::Pause,
    )
    .await?;
    assert_eq!(pause.status, RecordingStatus::Paused);
    assert!(
        !stores
            .recordings
            .capture_report(&initial_intent, &observation)
            .await?,
        "stale acknowledgement is fenced"
    );
    let paused_intent = stores
        .recordings
        .capture_intents(Some(session.id))
        .await?
        .remove(0);
    assert!(
        stores
            .recordings
            .capture_report(
                &paused_intent,
                &mahfel_postgres::CaptureObservation {
                    capturing: false,
                    sealed: true,
                    ..observation
                }
            )
            .await?
    );
    let paused_message = Uuid::new_v4();
    sqlx::query("INSERT INTO mahfel.chat_messages (id,channel_id,author_actor_id,body,created_at,session_id) VALUES ($1,$2,$3,'paused',clock_timestamp(),$4)")
        .bind(paused_message).bind(channel).bind(manager.id.as_uuid()).bind(session.id.as_uuid()).execute(&pool).await?;
    sqlx::query("INSERT INTO mahfel.collaboration_events (session_id,event_sequence,id,actor_id,document_id,kind,update_bytes) VALUES ($1,3,$2,$3,$4,'document_update',$5)")
        .bind(session.id.as_uuid()).bind(Uuid::new_v4()).bind(manager.id.as_uuid()).bind(document).bind(vec![0_u8,0]).execute(&pool).await?;
    let resumed = stores
        .recordings
        .command(
            session.id,
            manager.id,
            Uuid::new_v4(),
            RecordingAction::Resume,
        )
        .await?;
    assert_eq!(resumed.id, start.id);
    assert_eq!(resumed.capture_generation, 2);
    sqlx::query("UPDATE mahfel.chat_messages SET body = 'edited', edited_at = clock_timestamp() WHERE id = ANY($1)")
        .bind(vec![message, paused_message]).execute(&pool).await?;
    sqlx::query("UPDATE mahfel.chat_messages SET deleted_at = clock_timestamp() WHERE id = $1")
        .bind(message)
        .execute(&pool)
        .await?;
    let chat_events: i64 = sqlx::query_scalar("SELECT count(*) FROM mahfel.recording_events WHERE recording_id = $1 AND event_kind = 'chat'")
        .bind(start.id.as_uuid()).fetch_one(&pool).await?;
    assert_eq!(
        chat_events, 3,
        "create/edit/delete retained, paused chat and its later edit excluded"
    );
    let collaboration_events: i64 = sqlx::query_scalar("SELECT count(*) FROM mahfel.recording_events WHERE recording_id = $1 AND event_kind = 'collaboration'")
        .bind(start.id.as_uuid()).fetch_one(&pool).await?;
    assert_eq!(
        collaboration_events, 1,
        "paused collaboration events excluded"
    );
    let resumed_updates: i64 = sqlx::query_scalar("SELECT count(*) FROM mahfel.recording_snapshots WHERE recording_id = $1 AND generation = 2 AND snapshot_kind = 'document_update'")
        .bind(start.id.as_uuid()).fetch_one(&pool).await?;
    assert_eq!(
        resumed_updates, 3,
        "resume baseline includes workspace changes made while paused"
    );
    let generation_two = stores
        .recordings
        .capture_intents(Some(session.id))
        .await?
        .into_iter()
        .find(|intent| intent.generation == 2)
        .expect("generation two");
    assert!(
        stores
            .recordings
            .capture_report(
                &generation_two,
                &mahfel_postgres::CaptureObservation {
                    boot,
                    origin: 1000,
                    capturing: true,
                    sealed: false,
                    warning: None
                }
            )
            .await?
    );
    let known_two = stores
        .recordings
        .capture_intents(Some(session.id))
        .await?
        .into_iter()
        .find(|intent| intent.generation == 2)
        .expect("generation two");
    let next_boot = Uuid::new_v4();
    assert!(
        stores
            .recordings
            .capture_report(
                &known_two,
                &mahfel_postgres::CaptureObservation {
                    boot: next_boot,
                    origin: 2000,
                    capturing: true,
                    sealed: false,
                    warning: None
                }
            )
            .await?
    );
    let recovered = stores.recordings.list(session.id).await?.remove(0);
    assert_eq!(recovered.capture_generation, 3);
    assert!(recovered.partial);
    assert!(
        !stores
            .recordings
            .capture_report(
                &known_two,
                &mahfel_postgres::CaptureObservation {
                    boot,
                    origin: 1000,
                    capturing: true,
                    sealed: false,
                    warning: None
                }
            )
            .await?
    );
    session.end(OffsetDateTime::now_utc())?;
    stores.sessions.save(&session).await?;
    let stopped = stores.recordings.list(session.id).await?;
    assert_eq!(stopped[0].status, RecordingStatus::Stopping);
    assert!(stopped[0].stopped_at.is_some());
    let terminal = stores
        .recordings
        .capture_intents(Some(session.id))
        .await?
        .into_iter()
        .find(|intent| intent.generation == 3)
        .expect("terminal generation");
    assert!(
        stores
            .recordings
            .capture_report(
                &terminal,
                &mahfel_postgres::CaptureObservation {
                    boot: next_boot,
                    origin: 2000,
                    capturing: false,
                    sealed: true,
                    warning: None
                }
            )
            .await?
    );
    assert!(
        stores
            .recordings
            .finalize_sealed(start.id.as_uuid())
            .await?
    );
    assert!(
        !stores
            .recordings
            .finalize_sealed(start.id.as_uuid())
            .await?,
        "job publication is idempotent"
    );
    let job_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM mahfel.background_jobs WHERE job_type = 'recordings.finalize'",
    )
    .fetch_one(&pool)
    .await?;
    let jobs = mahfel_postgres::JobStore::new(pool.clone());
    let job = jobs
        .inspect(job_id)
        .await?
        .expect("finalization job")
        .envelope;
    assert_eq!(job.routing_key(), "recordings.finalize.v1");
    let claim = jobs.claim(&job).await?;
    let mahfel_postgres::JobClaim::Claimed(token) = claim else {
        panic!("expected job claim")
    };
    use mahfel_application::ports::{ManifestAcceptance, RecordingFinalizationStore};
    let acceptance = ManifestAcceptance {
        recording: start.id.as_uuid(),
        generation: 3,
        key: "recordings/test/manifest.json".into(),
        playback_key: "recordings/test/playback.json".into(),
        partial: true,
        usable: true,
    };
    assert!(
        !stores
            .recordings
            .accept(&job, Uuid::new_v4(), &acceptance)
            .await?
    );
    assert!(stores.recordings.accept(&job, token, &acceptance).await?);
    assert_eq!(
        stores.recordings.list(session.id).await?[0].status,
        RecordingStatus::Ready
    );
    assert!(
        !stores
            .files
            .recording_pinned(mahfel_domain::FileId::from_uuid(asset))
            .await?
    );
    assert!(jobs.succeed(&job, token).await?);
    {
        use mahfel_application::ports::{ExportArtifact, RecordingExportStore};
        use mahfel_domain::RecordingExportStatus;
        let exported =
            RecordingExportStore::get(&stores.recordings, session.id, start.id.as_uuid())
                .await?
                .expect("automatic export");
        assert_eq!(exported.status, RecordingExportStatus::Queued);
        assert!(
            stores
                .recordings
                .can_download(session.id, member.id)
                .await?
        );
        assert!(
            !stores
                .recordings
                .can_download(session.id, outsider.id)
                .await?
        );
        let block_id = Uuid::new_v4();
        sqlx::query("INSERT INTO mahfel.blocks (id,session_id,actor_id,blocked_by,blocked_at) VALUES ($1,$2,$3,$4,clock_timestamp())").bind(block_id).bind(session.id.as_uuid()).bind(member.id.as_uuid()).bind(manager.id.as_uuid()).execute(&pool).await?;
        assert!(
            !stores
                .recordings
                .can_download(session.id, member.id)
                .await?,
            "active blocks revoke download access"
        );
        sqlx::query(
            "UPDATE mahfel.blocks SET lifted_at = clock_timestamp(), lifted_by = $2 WHERE id = $1",
        )
        .bind(block_id)
        .bind(manager.id.as_uuid())
        .execute(&pool)
        .await?;
        assert!(
            stores
                .recordings
                .can_download(session.id, member.id)
                .await?
        );
        sqlx::query("UPDATE mahfel.rooms SET visibility = 'public' WHERE id = $1")
            .bind(room.id.as_uuid())
            .execute(&pool)
            .await?;
        assert!(
            !stores
                .recordings
                .can_download(session.id, outsider.id)
                .await?,
            "public visibility grants no recording download"
        );
        let export_job_id: Uuid =
            sqlx::query_scalar("SELECT job_id FROM mahfel.recording_exports WHERE id = $1")
                .bind(exported.id)
                .fetch_one(&pool)
                .await?;
        let export_job = jobs
            .inspect(export_job_id)
            .await?
            .expect("export outbox job")
            .envelope;
        let mahfel_postgres::JobClaim::Claimed(export_token) = jobs.claim(&export_job).await?
        else {
            panic!("export claim")
        };
        assert!(
            stores
                .recordings
                .begin(&export_job, Uuid::new_v4())
                .await?
                .is_none()
        );
        let source = stores
            .recordings
            .begin(&export_job, export_token)
            .await?
            .expect("fenced export");
        assert!(
            !stores
                .recordings
                .export_progress(exported.id, Uuid::new_v4(), 75)
                .await?
        );
        assert!(
            stores
                .recordings
                .export_progress(exported.id, export_token, 75)
                .await?
        );
        let artifact = ExportArtifact {
            object_key: format!(
                "recordings/{}/exports/{}/{}-{export_token}.mp4",
                source.recording_id, source.export_id, source.revision
            ),
            duration_ms: 1000,
            size_bytes: 100,
            sha256: "a".repeat(64),
        };
        assert!(
            !RecordingExportStore::accept(
                &stores.recordings,
                &export_job,
                Uuid::new_v4(),
                &artifact
            )
            .await?
        );
        assert!(
            jobs.fail(&export_job, export_token, "fixture_export_failure", true)
                .await?
        );
        assert_eq!(
            stores.recordings.list(session.id).await?[0].status,
            RecordingStatus::Ready,
            "export failure preserves original recording"
        );
        assert_eq!(
            RecordingExportStore::get(&stores.recordings, session.id, start.id.as_uuid())
                .await?
                .expect("failed export")
                .status,
            RecordingExportStatus::Failed
        );
        let retry = stores
            .recordings
            .retry(session.id, start.id.as_uuid())
            .await?;
        assert_eq!(retry.status, RecordingExportStatus::Queued);
        assert_eq!(retry.id, exported.id);
        assert!(
            !RecordingExportStore::accept(&stores.recordings, &export_job, export_token, &artifact)
                .await?
        );
        let next_job_id: Uuid =
            sqlx::query_scalar("SELECT job_id FROM mahfel.recording_exports WHERE id = $1")
                .bind(exported.id)
                .fetch_one(&pool)
                .await?;
        assert_ne!(next_job_id, export_job_id);
        stores
            .recordings
            .retry(session.id, start.id.as_uuid())
            .await?;
        let same_job_id: Uuid =
            sqlx::query_scalar("SELECT job_id FROM mahfel.recording_exports WHERE id = $1")
                .bind(exported.id)
                .fetch_one(&pool)
                .await?;
        assert_eq!(next_job_id, same_job_id, "queued retries are idempotent");
        let next_job = jobs
            .inspect(next_job_id)
            .await?
            .expect("retry export job")
            .envelope;
        let mahfel_postgres::JobClaim::Claimed(next_token) = jobs.claim(&next_job).await? else {
            panic!("retry claim")
        };
        let next_source = stores
            .recordings
            .begin(&next_job, next_token)
            .await?
            .expect("retry begin");
        let next_artifact = ExportArtifact {
            object_key: format!(
                "recordings/{}/exports/{}/{}-{next_token}.mp4",
                next_source.recording_id, next_source.export_id, next_source.revision
            ),
            ..artifact
        };
        assert!(
            RecordingExportStore::accept(&stores.recordings, &next_job, next_token, &next_artifact)
                .await?
        );
        assert!(jobs.succeed(&next_job, next_token).await?);
        assert!(
            !RecordingExportStore::accept(
                &stores.recordings,
                &next_job,
                next_token,
                &next_artifact
            )
            .await?
        );
        assert_eq!(
            RecordingExportStore::get(&stores.recordings, session.id, start.id.as_uuid())
                .await?
                .expect("ready export")
                .status,
            RecordingExportStatus::Ready
        );
        sqlx::query("UPDATE mahfel.recording_export_attempts SET created_at = clock_timestamp() - INTERVAL '2 days'").execute(&pool).await?;
        let orphaned = stores.recordings.export_orphans().await?;
        assert!(orphaned.iter().any(|(token, _)| *token == export_token));
        assert!(
            !orphaned.iter().any(|(token, _)| *token == next_token),
            "accepted object is protected from orphan cleanup"
        );
        stores
            .recordings
            .finish_export_cleanup(export_token)
            .await?;
    }
    let audit_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM mahfel.recording_audit WHERE recording_id = $1")
            .bind(start.id.as_uuid())
            .fetch_one(&pool)
            .await?;
    assert_eq!(audit_count, 4);
    assert!(
        sqlx::query("UPDATE mahfel.recording_audit SET reason = 'manual' WHERE recording_id = $1")
            .bind(start.id.as_uuid())
            .execute(&pool)
            .await
            .is_err()
    );
    let mut scheduled = Session::schedule(room.id, None, OffsetDateTime::now_utc());
    stores.sessions.insert(&scheduled).await?;
    scheduled.start(OffsetDateTime::now_utc())?;
    stores.sessions.save(&scheduled).await?;
    let automatic = stores.recordings.list(scheduled.id).await?;
    assert_eq!(automatic.len(), 1);
    assert_eq!(automatic[0].initiated_by, None);
    let first_join_document = Uuid::new_v4();
    sqlx::query("INSERT INTO mahfel.collaboration_documents (id, session_id, kind, schema_version) VALUES ($1,$2,'notes',1)")
        .bind(first_join_document).bind(scheduled.id.as_uuid()).execute(&pool).await?;
    let document_baseline: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM mahfel.recording_snapshots WHERE recording_id = $1 AND snapshot_kind = 'document' AND source_id = $2)")
        .bind(automatic[0].id.as_uuid()).bind(first_join_document).fetch_one(&pool).await?;
    assert!(
        document_baseline,
        "documents created after auto-start retain their definition"
    );
    scheduled.auto_record = false;
    assert!(
        stores
            .sessions
            .save_details(
                &scheduled,
                SessionDetailsFields {
                    auto_record: true,
                    ..Default::default()
                }
            )
            .await
            .is_err()
    );
    assert!(
        stores
            .sessions
            .get(scheduled.id)
            .await?
            .expect("session")
            .auto_record
    );
    scheduled.end(OffsetDateTime::now_utc())?;
    stores.sessions.save(&scheduled).await?;
    let terminal = stores
        .recordings
        .capture_intents(Some(scheduled.id))
        .await?
        .remove(0);
    stores
        .recordings
        .capture_report(
            &terminal,
            &mahfel_postgres::CaptureObservation {
                boot: Uuid::new_v4(),
                origin: 3000,
                capturing: false,
                sealed: true,
                warning: None,
            },
        )
        .await?;
    stores
        .recordings
        .finalize_sealed(automatic[0].id.as_uuid())
        .await?;
    let failed_job_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM mahfel.background_jobs WHERE payload ->> 'recording_id' = $1",
    )
    .bind(automatic[0].id.to_string())
    .fetch_one(&pool)
    .await?;
    let failed_job = jobs.inspect(failed_job_id).await?.expect("job").envelope;
    let mahfel_postgres::JobClaim::Claimed(failed_token) = jobs.claim(&failed_job).await? else {
        panic!("expected failure claim")
    };
    assert!(
        jobs.fail(&failed_job, failed_token, "fixture_permanent", true)
            .await?
    );
    assert_eq!(
        stores.recordings.list(scheduled.id).await?[0].status,
        RecordingStatus::Failed
    );
    assert!(jobs.replay_dead(failed_job_id).await?);
    let replay = jobs
        .inspect(failed_job_id)
        .await?
        .expect("replayed job")
        .envelope;
    assert_eq!(replay.generation, failed_job.generation + 1);
    assert_eq!(
        stores.recordings.list(scheduled.id).await?[0].status,
        RecordingStatus::Finalizing
    );
    let mahfel_postgres::JobClaim::Claimed(replay_token) = jobs.claim(&replay).await? else {
        panic!("expected replay claim")
    };
    let replay_acceptance = ManifestAcceptance {
        recording: automatic[0].id.as_uuid(),
        generation: 1,
        key: "recordings/test/replayed-manifest.json".into(),
        playback_key: "recordings/test/replayed-playback.json".into(),
        partial: false,
        usable: true,
    };
    assert!(
        !stores
            .recordings
            .accept(&failed_job, failed_token, &replay_acceptance)
            .await?
    );
    assert!(
        stores
            .recordings
            .accept(&replay, replay_token, &replay_acceptance)
            .await?
    );
    assert!(jobs.succeed(&replay, replay_token).await?);
    // Apply the migration's backfill to a legacy Ready recording. It must keep
    // the accepted original manifest and the existing export while adding replay.
    sqlx::query("UPDATE mahfel.recordings SET playback_index_key = NULL WHERE id = $1")
        .bind(automatic[0].id.as_uuid())
        .execute(&pool)
        .await?;
    let migration: &'static str =
        include_str!("../../../migrations/0033_recorded_session_start.sql");
    let backfill: &'static str =
        &migration[migration.find("WITH legacy AS").expect("backfill SQL")..];
    sqlx::raw_sql(backfill).execute(&pool).await?;
    let backfill_id: Uuid =
        sqlx::query_scalar("SELECT id FROM mahfel.background_jobs WHERE idempotency_key = $1")
            .bind(format!("recording.playback.backfill:{}:1", automatic[0].id))
            .fetch_one(&pool)
            .await?;
    let backfill_job = jobs
        .inspect(backfill_id)
        .await?
        .expect("backfill job")
        .envelope;
    let mahfel_postgres::JobClaim::Claimed(backfill_token) = jobs.claim(&backfill_job).await?
    else {
        panic!("backfill claim")
    };
    let backfill_acceptance = ManifestAcceptance {
        key: "recordings/test/new-derived-manifest.json".into(),
        playback_key: "recordings/test/backfilled-playback.json".into(),
        ..replay_acceptance
    };
    assert!(
        stores
            .recordings
            .accept(&backfill_job, backfill_token, &backfill_acceptance)
            .await?
    );
    assert!(jobs.succeed(&backfill_job, backfill_token).await?);
    let (original, playback): (String, String) = sqlx::query_as(
        "SELECT manifest_key, playback_index_key FROM mahfel.recordings WHERE id = $1",
    )
    .bind(automatic[0].id.as_uuid())
    .fetch_one(&pool)
    .await?;
    assert_eq!(original, "recordings/test/replayed-manifest.json");
    assert_eq!(playback, "recordings/test/backfilled-playback.json");
    let exports: i64 =
        sqlx::query_scalar("SELECT count(*) FROM mahfel.recording_exports WHERE recording_id = $1")
            .bind(automatic[0].id.as_uuid())
            .fetch_one(&pool)
            .await?;
    assert_eq!(exports, 1, "backfill must not duplicate exports");
    {
        use mahfel_application::ports::RecordingPlaybackStore;
        let recording = automatic[0].id.as_uuid();
        stores
            .recordings
            .delete(scheduled.id, recording, manager.id)
            .await?;
        assert!(
            stores.recordings.list(scheduled.id).await?.is_empty(),
            "deleted recordings disappear from the list"
        );
        assert!(
            stores
                .recordings
                .manifest(scheduled.id, recording)
                .await?
                .is_none(),
            "deleted recordings cannot be replayed"
        );
        let deleted = stores.recordings.deleted_recordings().await?;
        assert!(
            deleted.contains(&recording),
            "deletion is queued for cleanup"
        );
        stores
            .recordings
            .finish_recording_cleanup(recording)
            .await?;
        let retained_fragments: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM mahfel.recording_fragments WHERE recording_id = $1",
        )
        .bind(recording)
        .fetch_one(&pool)
        .await?;
        let retained_exports: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM mahfel.recording_exports WHERE recording_id = $1",
        )
        .bind(recording)
        .fetch_one(&pool)
        .await?;
        assert_eq!(retained_fragments, 0, "cleanup removes bundle artifacts");
        assert_eq!(retained_exports, 0, "cleanup removes export artifacts");
        let cleaned: bool = sqlx::query_scalar(
            "SELECT cleanup_checked_at IS NOT NULL FROM mahfel.recordings WHERE id = $1",
        )
        .bind(recording)
        .fetch_one(&pool)
        .await?;
        assert!(cleaned, "cleanup completion is recorded");
        let deletions: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM mahfel.recording_audit WHERE recording_id = $1 AND action_kind = 'delete'",
        )
        .bind(recording)
        .fetch_one(&pool)
        .await?;
        assert_eq!(deletions, 1, "manual deletion is audited");
    }
    pool.close().await;
    Ok(())
}
