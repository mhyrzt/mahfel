use anyhow::{Context, Result};
use mahfel_application::files::cleanup_expired;
use mahfel_application::ports::{FileStorage, RecordingStorage};
use mahfel_config::Settings;
use mahfel_postgres::{ConnectionSettings, JobStore, PoolSettings, PostgresFileStore};
use mahfel_storage::{S3FileStorage, S3Settings};
use mahfel_telemetry::{
    HealthState, init_tracing, operations, serve_http, shutdown_signal, spawn_probe,
};
use std::{net::SocketAddr, time::Duration};
use tokio::{net::TcpListener, sync::watch};

mod jobs;
mod recording_export;

const SERVICE: &str = "mahfel-worker";
const CLEANUP_INTERVAL: Duration = Duration::from_secs(300);

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::args().nth(1).as_deref() == Some("render-export") {
        let directory = std::env::args()
            .nth(2)
            .context("expected export workspace")?;
        let renderer = std::env::args()
            .nth(3)
            .context("expected renderer directory")?;
        return recording_export::render_local(
            std::path::Path::new(&directory),
            std::path::Path::new(&renderer),
        )
        .await;
    }
    let settings = Settings::load().context("load configuration")?;
    anyhow::ensure!(
        settings.worker.exports.concurrency > 0
            && settings.worker.exports.concurrency <= usize::from(u16::MAX)
            && settings.worker.exports.max_active_audio_tracks > 0
            && settings.worker.exports.output_bytes > 0
            && settings.worker.exports.scratch_bytes
                > settings.worker.exports.output_bytes.saturating_mul(2)
            && settings.worker.exports.timeout_seconds > 0
            && settings.worker.concurrency > 0
            && settings.worker.prefetch > 0
            && settings.worker.drain_timeout_seconds > 0,
        "worker concurrency, prefetch, timeout and export resource limits must be valid"
    );
    init_tracing(&settings.telemetry.log_filter).context("initialize tracing")?;
    if let Some(command) = std::env::args().nth(1) {
        return run_command(&settings, &command).await;
    }
    let (operations_router, health) = operations(SERVICE, env!("CARGO_PKG_VERSION"))?;
    health.set_check("rabbitmq", false).await;
    health.set_check("database", false).await;
    health.set_initialized(true).await;
    let metrics = jobs::JobMetrics::register(&health.registry())?;
    let operations_address = settings.worker.operations_bind.parse::<SocketAddr>()?;
    let listener = TcpListener::bind(operations_address).await?;
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let services_task = tokio::spawn(services_loop(
        settings,
        health.clone(),
        metrics,
        shutdown_rx.clone(),
    ));
    let operations_task = tokio::spawn(serve_http(listener, operations_router, shutdown_rx));

    tracing::info!(%operations_address, "worker operations listener ready");
    shutdown_signal().await;
    health.set_initialized(false).await;
    let _ = shutdown_tx.send(true);
    operations_task
        .await
        .context("join operations listener")??;
    services_task.await.context("join worker services")?;
    Ok(())
}

async fn connect_database(settings: &Settings) -> Result<sqlx::PgPool> {
    mahfel_postgres::connect(
        &ConnectionSettings {
            host: settings.database.host.clone(),
            port: settings.database.port,
            user: settings.database.user.clone(),
            password: settings.database.password.clone(),
            name: settings.database.name.clone(),
        },
        PoolSettings {
            min_connections: settings.database.min_connections,
            max_connections: settings.database.max_connections,
            acquire_timeout: Duration::from_secs(settings.database.acquire_timeout_seconds),
            idle_timeout: Duration::from_secs(settings.database.idle_timeout_seconds),
            max_lifetime: Duration::from_secs(settings.database.max_lifetime_seconds),
        },
    )
    .await
    .context("connect to PostgreSQL")
}

async fn run_command(settings: &Settings, command: &str) -> Result<()> {
    let action = std::env::args()
        .nth(2)
        .context("expected jobs inspect|replay")?;
    let id = std::env::args().nth(3).context("expected job UUID")?;
    anyhow::ensure!(command == "jobs", "expected jobs inspect|replay <job-id>");
    let database = connect_database(settings).await?;
    let id = uuid::Uuid::parse_str(&id).context("parse job UUID")?;
    let jobs = JobStore::new(database.clone());
    match action.as_str() {
        "inspect" => {
            let record = jobs.inspect(id).await?.context("job not found")?;
            println!(
                "job={} type={} status={} generation={} attempt={} last_failure={}",
                id,
                record.envelope.job_type,
                record.status,
                record.generation,
                record.envelope.attempt,
                record.last_failure_code.as_deref().unwrap_or("none")
            );
            for attempt in jobs.history(id).await? {
                println!(
                    "generation={} attempt={} outcome={} failure={} at={}",
                    attempt.generation,
                    attempt.attempt,
                    attempt.outcome,
                    attempt.failure_code.as_deref().unwrap_or("none"),
                    attempt.finished_at
                );
            }
            for replay in jobs.replays(id).await? {
                println!(
                    "replayed generation={} at={}",
                    replay.generation, replay.replayed_at
                );
            }
        }
        "replay" => {
            anyhow::ensure!(jobs.replay_dead(id).await?, "job is missing or not dead");
            println!("replayed job {id}");
        }
        _ => anyhow::bail!("expected jobs inspect|replay <job-id>"),
    }
    database.close().await;
    Ok(())
}

async fn services_loop(
    settings: Settings,
    health: HealthState,
    metrics: jobs::JobMetrics,
    mut shutdown: watch::Receiver<bool>,
) {
    let database = loop {
        if *shutdown.borrow() {
            return;
        }
        match connect_database(&settings).await {
            Ok(database) => break database,
            Err(error) => tracing::warn!(%error, "worker database unavailable; retrying"),
        }
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(5)) => {},
            _ = shutdown.changed() => return,
        }
    };
    health.set_check("database", true).await;
    let files = PostgresFileStore::new(database.clone());
    let storage = S3FileStorage::new(S3Settings::from(&settings.storage));
    let probe_pool = database.clone();
    let db_probe = spawn_probe(
        "database",
        Duration::from_secs(5),
        health.clone(),
        shutdown.clone(),
        move || {
            let pool = probe_pool.clone();
            async move { mahfel_postgres::ping(&pool).await.is_ok() }
        },
    );
    let cleanup_task = tokio::spawn(file_cleanup_loop(
        files.clone(),
        storage.clone(),
        mahfel_postgres::PostgresRecordingStore::new(database.clone()),
        settings.worker.exports.scratch_directory.clone(),
        shutdown.clone(),
    ));
    let jobs_task = tokio::spawn(jobs::run(
        settings.worker.clone(),
        JobStore::new(database.clone()),
        files,
        storage,
        health.clone(),
        metrics,
        shutdown.clone(),
    ));
    let _ = shutdown.changed().await;
    let _ = jobs_task.await;
    let _ = cleanup_task.await;
    let _ = db_probe.await;
    database.close().await;
}

async fn delete_prefix(storage: &S3FileStorage, prefix: &str) -> Result<bool> {
    let mut continuation = None;
    let mut complete = true;
    loop {
        let page = RecordingStorage::page(storage, prefix, continuation)
            .await
            .context("list recording objects")?;
        for key in page.keys {
            if FileStorage::delete(storage, &key).await.is_err() {
                complete = false;
            }
        }
        continuation = page.continuation;
        if continuation.is_none() {
            break;
        }
    }
    Ok(complete)
}

async fn file_cleanup_loop(
    files: PostgresFileStore,
    storage: S3FileStorage,
    recordings: mahfel_postgres::PostgresRecordingStore,
    scratch: std::path::PathBuf,
    mut shutdown: watch::Receiver<bool>,
) {
    let mut interval = tokio::time::interval(CLEANUP_INTERVAL);
    loop {
        tokio::select! {
            _ = interval.tick() => {
                if let Err(error) = recording_export::cleanup_scratch(&scratch, &recordings).await {
                    tracing::warn!(%error, "export scratch cleanup unavailable");
                }
                match recordings.export_orphans().await {
                    Ok(attempts) => for (token, key) in attempts {
                        if mahfel_application::ports::FileStorage::delete(&storage, &key).await.is_ok() { let _ = recordings.finish_export_cleanup(token).await; }
                    },
                    Err(error) => tracing::warn!(%error, "export orphan cleanup unavailable"),
                }
                match cleanup_expired(&files, &storage).await {
                    Ok(count) if count > 0 => tracing::info!(count, "expired file uploads removed"),
                    Ok(_) => {},
                    Err(error) => tracing::warn!(%error, "file cleanup failed"),
                }
                match recordings.deleted_recordings().await {
                    Ok(deleted) => for recording in deleted {
                        let prefix = format!("recordings/{recording}/");
                        match delete_prefix(&storage, &prefix).await {
                            Ok(true) => match recordings.finish_recording_cleanup(recording).await {
                                Ok(()) => tracing::info!(%recording, "deleted recording artifacts removed"),
                                Err(error) => tracing::warn!(%error, %recording, "recording cleanup finalize failed"),
                            },
                            Ok(false) => tracing::warn!(%recording, "recording storage cleanup incomplete; retrying"),
                            Err(error) => tracing::warn!(%error, %recording, "recording storage cleanup unavailable"),
                        }
                    },
                    Err(error) => tracing::warn!(%error, "deleted recording cleanup unavailable"),
                }
            }
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() { break; }
            }
        }
    }
}
