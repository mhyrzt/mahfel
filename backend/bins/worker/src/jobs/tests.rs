use super::{JobMetrics, run};
use anyhow::Context;
use aws_sdk_s3::{
    Client,
    config::{BehaviorVersion, Config, Credentials, Region},
};
use mahfel_application::ports::{ChatStore, FileStore, RoomStore, UserStore};
use mahfel_config::WorkerSettings;
use mahfel_domain::{
    ChatChannel, ChatChannelId, ChatMessageDraft, ChatMessageId, DisplayName, EmailAddress,
    FileContext, FileObject, Room, RoomMember, RoomRole, RoomTitle, RoomVisibility, User, Username,
};
use mahfel_postgres::{
    ConnectionSettings, PoolSettings, PostgresChatStore, PostgresRoomStore, PostgresUserStore,
    connect as connect_postgres, migrate,
};
use mahfel_postgres::{JobStore, PostgresFileStore};
use mahfel_storage::S3FileStorage;
use mahfel_storage::S3Settings;
use std::error::Error;
use std::time::Duration;
use testcontainers::{
    GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};
use time::OffsetDateTime;
use tokio::sync::watch;
use uuid::Uuid;

#[tokio::test]
async fn chat_deletion_drains_outbox_and_cleans_object() -> Result<(), Box<dyn Error>> {
    if std::env::var("MAHFEL_TESTCONTAINERS").as_deref() != Ok("1") {
        eprintln!("MAHFEL_TESTCONTAINERS is unset; skipping worker integration test");
        return Ok(());
    }
    let _ = mahfel_telemetry::init_tracing("info");
    let postgres = GenericImage::new("postgres", "16-bookworm")
        .with_exposed_port(5432.tcp())
        .with_wait_for(WaitFor::message_on_stdout(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_DB", "mahfel")
        .with_env_var("POSTGRES_USER", "mahfel")
        .with_env_var("POSTGRES_PASSWORD", "mahfel-test")
        .start()
        .await?;
    let pg_port = postgres.get_host_port_ipv4(5432).await?;
    let database = tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            let connected = connect_postgres(
                &ConnectionSettings {
                    host: "127.0.0.1".into(),
                    port: pg_port,
                    user: "mahfel".into(),
                    password: "mahfel-test".into(),
                    name: "mahfel".into(),
                },
                PoolSettings {
                    min_connections: 1,
                    max_connections: 5,
                    acquire_timeout: Duration::from_secs(5),
                    idle_timeout: Duration::from_secs(30),
                    max_lifetime: Duration::from_secs(60),
                },
            )
            .await;
            match connected {
                Ok(pool) => break pool,
                Err(_) => tokio::time::sleep(Duration::from_millis(200)).await,
            }
        }
    })
    .await
    .context("connect test postgres")?;
    migrate(&database).await?;
    let rustfs = GenericImage::new("docker.arvancloud.ir/rustfs/rustfs", "1.0.0-rc.6")
        .with_exposed_port(9000.tcp())
        .with_wait_for(WaitFor::seconds(3))
        .with_env_var("RUSTFS_VOLUMES", "/data")
        .with_env_var("RUSTFS_ADDRESS", "0.0.0.0:9000")
        .with_env_var("RUSTFS_ACCESS_KEY", "mahfel-test")
        .with_env_var("RUSTFS_SECRET_KEY", "mahfel-test-secret")
        .start()
        .await?;
    let s3_port = rustfs.get_host_port_ipv4(9000).await?;
    let endpoint = format!("http://127.0.0.1:{s3_port}");
    let bucket = format!("mahfel-job-test-{}", Uuid::new_v4().simple());
    let s3_settings = S3Settings {
        internal_endpoint: endpoint.clone(),
        public_endpoint: endpoint.clone(),
        region: "us-east-1".into(),
        bucket: bucket.clone(),
        access_key: "mahfel-test".into(),
        secret_key: "mahfel-test-secret".into(),
        force_path_style: true,
    };
    let config = Config::builder()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new("us-east-1"))
        .credentials_provider(Credentials::new(
            "mahfel-test",
            "mahfel-test-secret",
            None,
            None,
            "test",
        ))
        .endpoint_url(endpoint)
        .force_path_style(true)
        .build();
    let s3 = Client::from_conf(config);
    let mut last_bucket_error = None;
    for _ in 0..20 {
        match s3.create_bucket().bucket(&bucket).send().await {
            Ok(_) => {
                last_bucket_error = None;
                break;
            }
            Err(error) => {
                last_bucket_error = Some(error);
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
    if let Some(error) = last_bucket_error {
        return Err(error.into());
    }
    let now = OffsetDateTime::now_utc();
    let users = PostgresUserStore::new(database.clone());
    let rooms = PostgresRoomStore::new(database.clone());
    let chat = PostgresChatStore::new(database.clone());
    let files = PostgresFileStore::new(database.clone());
    let user = User::create(
        DisplayName::new("Job Test".into())?,
        EmailAddress::new("job@example.test".into())?,
        Username::new("job-test".into())?,
        now,
    );
    users.insert(&user).await?;
    let room = Room::create(
        RoomTitle::new("Job room".into())?,
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
    let message = chat
        .create_message(
            &ChatChannel::for_room(ChatChannelId::new(), room.id, now),
            &ChatMessageDraft::new(
                ChatMessageId::new(),
                user.id,
                "Delete attachment".into(),
                now,
            )?,
        )
        .await?;
    let file = FileObject::pending(
        user.id,
        FileContext::ChatMessage(message.id),
        "test.pdf".into(),
        "application/pdf".into(),
        8,
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".into(),
        now,
    )?;
    files.insert(&file).await?;
    s3.put_object()
        .bucket(&bucket)
        .key(&file.object_key)
        .body(b"testdata".to_vec().into())
        .send()
        .await?;
    chat.delete_message(room.id, user.id, message.id, now)
        .await?;
    let job_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM mahfel.background_jobs WHERE payload ->> 'file_id' = $1",
    )
    .bind(file.id.to_string())
    .fetch_one(&database)
    .await?;
    // Capture receipts remain durable while RabbitMQ is unavailable.
    use mahfel_application::ports::{RecordingStorage, RecordingStore, SessionStore};
    let sessions = mahfel_postgres::PostgresSessionStore::new(database.clone());
    let recordings = mahfel_postgres::PostgresRecordingStore::new(database.clone());
    let mut session = mahfel_domain::Session::schedule(room.id, None, OffsetDateTime::now_utc());
    sessions.insert(&session).await?;
    session.start(OffsetDateTime::now_utc())?;
    sessions.save(&session).await?;
    let recording = recordings.list(session.id).await?.remove(0);
    let boot = Uuid::new_v4();
    let origin = u64::try_from(OffsetDateTime::now_utc().unix_timestamp_nanos())?;
    let intent = recordings
        .capture_intents(Some(session.id))
        .await?
        .remove(0);
    recordings
        .capture_report(
            &intent,
            &mahfel_postgres::CaptureObservation {
                boot,
                origin,
                capturing: true,
                sealed: false,
                warning: None,
            },
        )
        .await?;
    let fixture = encoded_audio_fixture()?;
    let original = std::fs::read(&fixture)?;
    use sha2::{Digest, Sha256};
    let key = format!(
        "recordings/{}/capture/1/{boot}/tracks/audio/000000.webm",
        recording.id
    );
    let recording_storage = S3FileStorage::new(s3_settings.clone());
    recording_storage.put_fragment(&key, &fixture).await?;
    std::fs::remove_file(fixture)?;
    let receipt = mahfel_application::recording_artifacts::FragmentReceipt {
        recording_id: recording.id.as_uuid(),
        generation: 1,
        boot_id: boot,
        producer_id: Uuid::new_v4().to_string(),
        actor_id: user.id.as_uuid(),
        source: "microphone".into(),
        codec: "audio/opus".into(),
        container: "webm".into(),
        unix_origin_ns: origin,
        start_ns: 0,
        end_ns: 1_200_000_000,
        object_key: key.clone(),
        sha256: format!("{:x}", Sha256::digest(&original)),
        bytes: original.len() as u64,
    };
    recording_storage
        .put(
            &format!(
                "recordings/{}/capture/1/{boot}/receipts/000000.json",
                recording.id
            ),
            serde_json::to_vec(&receipt)?,
        )
        .await?;
    // One corrupt track yields a usable partial bundle with an explicit loss.
    let corrupt_key = format!(
        "recordings/{}/capture/1/{boot}/tracks/corrupt/000000.webm",
        recording.id
    );
    let corrupt_bytes = b"corrupt-container".to_vec();
    recording_storage
        .put(&corrupt_key, corrupt_bytes.clone())
        .await?;
    let corrupt_receipt = mahfel_application::recording_artifacts::FragmentReceipt {
        producer_id: Uuid::new_v4().to_string(),
        object_key: corrupt_key,
        sha256: format!("{:x}", Sha256::digest(&corrupt_bytes)),
        bytes: corrupt_bytes.len() as u64,
        ..receipt.clone()
    };
    recording_storage
        .put(
            &format!(
                "recordings/{}/capture/1/{boot}/receipts/corrupt.json",
                recording.id
            ),
            serde_json::to_vec(&corrupt_receipt)?,
        )
        .await?;
    // A referenced file must survive source deletion after its private archive is accepted.
    let asset_id = Uuid::new_v4();
    let asset_key = format!("files/{asset_id}");
    let asset_bytes = b"archived-original".to_vec();
    recording_storage
        .put(&asset_key, asset_bytes.clone())
        .await?;
    use base64::Engine;
    let asset_checksum =
        base64::engine::general_purpose::STANDARD.encode(Sha256::digest(&asset_bytes));
    sqlx::query("INSERT INTO mahfel.files (id,object_key,owner_actor_id,session_id,status,file_name,declared_mime,verified_mime,size_bytes,checksum_sha_256,created_at,updated_at) VALUES ($1,$2,$3,$4,'ready','fixture.pdf','application/pdf','application/pdf',$5,$6,clock_timestamp(),clock_timestamp())")
        .bind(asset_id).bind(&asset_key).bind(user.id.as_uuid()).bind(session.id.as_uuid()).bind(asset_bytes.len() as i64).bind(&asset_checksum).execute(&database).await?;
    let document = Uuid::new_v4();
    sqlx::query("INSERT INTO mahfel.collaboration_documents (id,session_id,kind,schema_version) VALUES ($1,$2,'notes',1)")
        .bind(document).bind(session.id.as_uuid()).execute(&database).await?;
    sqlx::query(
        "INSERT INTO mahfel.collaboration_document_assets (document_id,file_id) VALUES ($1,$2)",
    )
    .bind(document)
    .bind(asset_id)
    .execute(&database)
    .await?;
    assert!(
        files
            .recording_pinned(mahfel_domain::FileId::from_uuid(asset_id))
            .await?
    );
    session.end(OffsetDateTime::now_utc())?;
    sessions.save(&session).await?;
    let intent = recordings
        .capture_intents(Some(session.id))
        .await?
        .remove(0);
    recordings
        .capture_report(
            &intent,
            &mahfel_postgres::CaptureObservation {
                boot,
                origin,
                capturing: false,
                sealed: true,
                warning: None,
            },
        )
        .await?;
    assert!(recordings.finalize_sealed(recording.id.as_uuid()).await?);
    let recording_job_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM mahfel.background_jobs WHERE job_type = 'recordings.finalize'",
    )
    .fetch_one(&database)
    .await?;
    let retry_store = JobStore::new(database.clone());
    let queued = retry_store
        .inspect(recording_job_id)
        .await?
        .expect("recording job")
        .envelope;
    let mahfel_postgres::JobClaim::Claimed(retry_token) = retry_store.claim(&queued).await? else {
        panic!("recording claim")
    };
    let unavailable_storage = S3FileStorage::new(S3Settings {
        internal_endpoint: "http://127.0.0.1:1".into(),
        ..s3_settings.clone()
    });
    let failed = mahfel_application::recordings::finalize_recording(
        &recordings,
        &unavailable_storage,
        &super::recording_verifier::GstreamerVerifier,
        &queued,
        retry_token,
    )
    .await;
    assert!(matches!(
        failed,
        Err(mahfel_application::jobs::JobFailure::Retryable(_))
    ));
    assert!(
        retry_store
            .fail(&queued, retry_token, "storage_fixture_outage", false)
            .await?
    );
    sqlx::query("UPDATE mahfel.job_outbox SET available_at = clock_timestamp() WHERE job_id = $1 AND attempt = 2")
        .bind(recording_job_id).execute(&database).await?;
    let recovery_store = JobStore::new(database.clone());
    let envelope = recovery_store
        .inspect(job_id)
        .await?
        .expect("cleanup job")
        .envelope;
    assert!(matches!(
        recovery_store.claim(&envelope).await?,
        mahfel_postgres::JobClaim::Claimed(_)
    ));
    sqlx::query("UPDATE mahfel.background_jobs SET lease_until = clock_timestamp() - INTERVAL '1 second' WHERE id = $1")
        .bind(job_id).execute(&database).await?;
    let (_, recovery_health) = mahfel_telemetry::operations("worker-recovery-test", "0")?;
    let recovery_metrics = JobMetrics::register(&recovery_health.registry())?;
    let (recovery_shutdown_tx, recovery_shutdown_rx) = watch::channel(false);
    let recovering_worker = tokio::spawn(run(
        WorkerSettings {
            exports: mahfel_config::RecordingExportSettings::default(),
            operations_bind: "127.0.0.1:0".into(),
            amqp_url: "amqp://127.0.0.1:1/%2f".into(),
            concurrency: 4,
            prefetch: 4,
            drain_timeout_seconds: 1,
        },
        recovery_store.clone(),
        files.clone(),
        S3FileStorage::new(s3_settings.clone()),
        recovery_health,
        recovery_metrics,
        recovery_shutdown_rx,
    ));
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if recovery_store
                .inspect(job_id)
                .await
                .expect("inspect recovering job")
                .expect("job")
                .status
                == "queued"
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .context("claim recovery must continue without RabbitMQ")?;
    recovery_shutdown_tx.send(true)?;
    tokio::time::timeout(Duration::from_secs(5), recovering_worker).await??;

    // RabbitMQ starts after the database commit: the outbox must bridge the outage.
    let broker = GenericImage::new("rabbitmq", "4.2-management-alpine")
        .with_exposed_port(5672.tcp())
        .with_wait_for(WaitFor::message_on_stdout("Server startup complete"))
        .with_env_var("RABBITMQ_DEFAULT_USER", "mahfel")
        .with_env_var("RABBITMQ_DEFAULT_PASS", "mahfel-test")
        .start()
        .await?;
    let amqp_port = broker.get_host_port_ipv4(5672).await?;
    let amqp_url = format!("amqp://mahfel:mahfel-test@127.0.0.1:{amqp_port}/%2f");
    let (_, health) = mahfel_telemetry::operations("worker-test", "0")?;
    let metrics = JobMetrics::register(&health.registry())?;
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let store = JobStore::new(database.clone());
    sqlx::query("CREATE TABLE mahfel.fixture_failed_completion (job_id UUID PRIMARY KEY)")
        .execute(&database)
        .await?;
    sqlx::query("INSERT INTO mahfel.fixture_failed_completion (job_id) VALUES ($1)")
        .bind(job_id)
        .execute(&database)
        .await?;
    sqlx::query("CREATE FUNCTION mahfel.fail_cleanup_completion() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.status = 'succeeded' AND EXISTS (SELECT 1 FROM mahfel.fixture_failed_completion WHERE job_id = NEW.id) THEN RAISE EXCEPTION 'fixture completion persistence failure'; END IF; RETURN NEW; END $$")
        .execute(&database).await?;
    sqlx::query("CREATE TRIGGER fail_cleanup_completion BEFORE UPDATE ON mahfel.background_jobs FOR EACH ROW EXECUTE FUNCTION mahfel.fail_cleanup_completion()")
        .execute(&database).await?;
    let worker = tokio::spawn(run(
        WorkerSettings {
            exports: mahfel_config::RecordingExportSettings::default(),
            operations_bind: "127.0.0.1:0".into(),
            amqp_url: amqp_url.clone(),
            concurrency: 4,
            prefetch: 4,
            drain_timeout_seconds: 30,
        },
        store.clone(),
        files.clone(),
        S3FileStorage::new(s3_settings),
        health,
        metrics,
        shutdown_rx,
    ));
    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            if files
                .get(file.id)
                .await
                .expect("cleanup metadata lookup")
                .is_none()
                && store
                    .inspect(recording_job_id)
                    .await
                    .expect("recording lookup")
                    .expect("recording job")
                    .status
                    == "succeeded"
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .context("another delivery must progress after cleanup completion persistence fails")?;
    assert!(
        !worker.is_finished(),
        "one failed delivery must not terminate the consumer"
    );
    assert_eq!(
        store.inspect(job_id).await?.expect("cleanup job").status,
        "running"
    );
    sqlx::query("DROP TRIGGER fail_cleanup_completion ON mahfel.background_jobs")
        .execute(&database)
        .await?;
    sqlx::query("UPDATE mahfel.background_jobs SET lease_until = clock_timestamp() - INTERVAL '1 second' WHERE id = $1")
        .bind(job_id).execute(&database).await?;
    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            let status = store
                .inspect(job_id)
                .await
                .expect("inspect job")
                .expect("job present");
            if status.status == "succeeded" {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .context("wait for cleanup job")?;
    assert!(files.get(file.id).await?.is_none(), "file metadata removed");
    assert!(
        s3.head_object()
            .bucket(&bucket)
            .key(&file.object_key)
            .send()
            .await
            .is_err(),
        "S3 object removed"
    );
    assert_eq!(store.history(job_id).await?.len(), 1);
    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            if store
                .inspect(recording_job_id)
                .await
                .expect("inspect recording job")
                .expect("recording job")
                .status
                == "succeeded"
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .context("wait for recording finalization")?;
    let accepted = recordings.list(session.id).await?.remove(0);
    assert_eq!(accepted.status, mahfel_domain::RecordingStatus::Ready);
    let manifest_key = accepted.manifest_key.expect("accepted manifest");
    let manifest: serde_json::Value =
        serde_json::from_slice(&recording_storage.read(&manifest_key, 1024 * 1024).await?)?;
    assert_eq!(manifest["version"], 1);
    assert_eq!(manifest["tracks"].as_object().expect("tracks").len(), 1);
    let playback_key: String =
        sqlx::query_scalar("SELECT playback_index_key FROM mahfel.recordings WHERE id = $1")
            .bind(recording.id.as_uuid())
            .fetch_one(&database)
            .await?;
    let playback: serde_json::Value =
        serde_json::from_slice(&recording_storage.read(&playback_key, 1024 * 1024).await?)?;
    assert_eq!(playback["version"], 1);
    let browser_segment = playback["segments"][0]["object_key"]
        .as_str()
        .expect("browser segment");
    let browser_bytes = recording_storage
        .read(browser_segment, 64 * 1024 * 1024)
        .await?;
    assert!(
        browser_bytes.windows(4).any(|window| window == b"ftyp"),
        "browser MP4 container"
    );
    assert!(manifest["partial"].as_bool().expect("partial coverage"));
    assert_eq!(manifest["losses"].as_array().expect("losses").len(), 1);
    assert_eq!(
        recording_storage.read(&key, 32 * 1024 * 1024).await?,
        original,
        "original encoded bytes preserved"
    );
    assert!(
        !files
            .recording_pinned(mahfel_domain::FileId::from_uuid(asset_id))
            .await?
    );
    s3.delete_object()
        .bucket(&bucket)
        .key(&asset_key)
        .send()
        .await?;
    assert_eq!(
        recording_storage
            .read(
                &format!("recordings/{}/assets/{asset_id}", recording.id),
                1024
            )
            .await?,
        asset_bytes
    );
    assert_eq!(
        store.history(recording_job_id).await?.len(),
        2,
        "durable retry followed by success"
    );
    // A duplicate finalization attempt accepts the existing durable result without rewriting it.
    let envelope = store
        .inspect(recording_job_id)
        .await?
        .expect("job")
        .envelope;
    mahfel_application::recordings::finalize_recording(
        &recordings,
        &recording_storage,
        &super::recording_verifier::GstreamerVerifier,
        &envelope,
        Uuid::new_v4(),
    )
    .await
    .map_err(|error| format!("duplicate finalization: {error:?}"))?;
    assert_eq!(
        recordings.list(session.id).await?[0]
            .manifest_key
            .as_deref(),
        Some(manifest_key.as_str())
    );
    let _ = shutdown_tx.send(true);
    tokio::time::timeout(Duration::from_secs(35), worker).await??;
    let mut empty_session =
        mahfel_domain::Session::schedule(room.id, None, OffsetDateTime::now_utc());
    sessions.insert(&empty_session).await?;
    empty_session.start(OffsetDateTime::now_utc())?;
    sessions.save(&empty_session).await?;
    let empty_recording = recordings.list(empty_session.id).await?.remove(0);
    empty_session.end(OffsetDateTime::now_utc())?;
    sessions.save(&empty_session).await?;
    let empty_intent = recordings
        .capture_intents(Some(empty_session.id))
        .await?
        .remove(0);
    recordings
        .capture_report(
            &empty_intent,
            &mahfel_postgres::CaptureObservation {
                boot: Uuid::new_v4(),
                origin: 1000,
                capturing: false,
                sealed: true,
                warning: None,
            },
        )
        .await?;
    recordings
        .finalize_sealed(empty_recording.id.as_uuid())
        .await?;
    let empty_job_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM mahfel.background_jobs WHERE payload ->> 'recording_id' = $1",
    )
    .bind(empty_recording.id.to_string())
    .fetch_one(&database)
    .await?;
    let empty_job = store
        .inspect(empty_job_id)
        .await?
        .expect("empty recording job")
        .envelope;
    let mahfel_postgres::JobClaim::Claimed(empty_token) = store.claim(&empty_job).await? else {
        return Err("empty recording claim missing".into());
    };
    mahfel_application::recordings::finalize_recording(
        &recordings,
        &recording_storage,
        &super::recording_verifier::GstreamerVerifier,
        &empty_job,
        empty_token,
    )
    .await
    .map_err(|error| format!("empty finalization: {error:?}"))?;
    let empty_result = recordings.list(empty_session.id).await?.remove(0);
    assert_eq!(empty_result.status, mahfel_domain::RecordingStatus::Failed);
    assert_eq!(empty_result.warning.as_deref(), Some("unusable_recording"));
    assert!(empty_result.manifest_key.is_none());
    assert!(store.succeed(&empty_job, empty_token).await?);
    database.close().await;
    Ok(())
}

fn encoded_audio_fixture() -> Result<std::path::PathBuf, Box<dyn Error>> {
    use gstreamer::{self as gst, prelude::*};
    gst::init()?;
    let path = std::env::temp_dir().join(format!("mahfel-original-{}.webm", Uuid::new_v4()));
    let pipeline = gst::parse::launch(
        "audiotestsrc num-buffers=50 ! audioconvert ! opusenc ! webmmux ! filesink name=output",
    )?
    .downcast::<gst::Pipeline>()
    .map_err(|_| "fixture pipeline type")?;
    pipeline
        .by_name("output")
        .ok_or("fixture output")?
        .set_property("location", path.to_str().ok_or("fixture path")?);
    pipeline.set_state(gst::State::Playing)?;
    let result = pipeline.bus().ok_or("fixture bus")?.timed_pop_filtered(
        gst::ClockTime::from_seconds(10),
        &[gst::MessageType::Eos, gst::MessageType::Error],
    );
    pipeline.set_state(gst::State::Null)?;
    if !result.is_some_and(|message| message.type_() == gst::MessageType::Eos) {
        return Err("fixture encoding failed".into());
    }
    Ok(path)
}
