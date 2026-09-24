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
    let worker = tokio::spawn(run(
        WorkerSettings {
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
    let _ = shutdown_tx.send(true);
    tokio::time::timeout(Duration::from_secs(35), worker).await??;
    database.close().await;
    Ok(())
}
