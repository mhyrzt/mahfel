use anyhow::{Context, Result};
use mahfel_application::files::cleanup_expired;
use mahfel_config::Settings;
use mahfel_messaging::connect;
use mahfel_postgres::{ConnectionSettings, PoolSettings, PostgresFileStore};
use mahfel_storage::{S3FileStorage, S3Settings};
use mahfel_telemetry::{init_tracing, operations, serve_http, shutdown_signal, spawn_probe};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{net::TcpListener, sync::watch};

const SERVICE: &str = "mahfel-worker";
const AMQP_PROBE: &str = "rabbitmq";
const PROBE_INTERVAL: Duration = Duration::from_secs(5);
const CLEANUP_INTERVAL: Duration = Duration::from_secs(300);

#[tokio::main]
async fn main() -> Result<()> {
    let settings = Settings::load().context("load configuration")?;
    init_tracing(&settings.telemetry.log_filter).context("initialize tracing")?;
    let connection = Arc::new(
        connect(&settings.worker.amqp_url)
            .await
            .context("connect to RabbitMQ")?,
    );
    let database = mahfel_postgres::connect(
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
    .context("connect to PostgreSQL for file cleanup")?;
    let files = PostgresFileStore::new(database.clone());
    let storage = S3FileStorage::new(S3Settings::from(&settings.storage));
    let (operations_router, health) = operations(SERVICE, env!("CARGO_PKG_VERSION"))?;
    health.set_check(AMQP_PROBE, true).await;
    health.set_initialized(true).await;

    let operations_address = settings.worker.operations_bind.parse::<SocketAddr>()?;
    let listener = TcpListener::bind(operations_address).await?;
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let cleanup_task = tokio::spawn(file_cleanup_loop(files, storage, shutdown_rx.clone()));
    let probe_task = spawn_probe(
        AMQP_PROBE,
        PROBE_INTERVAL,
        health.clone(),
        shutdown_rx.clone(),
        move || {
            let connection = Arc::clone(&connection);
            async move { connection.status().connected() }
        },
    );
    let operations_task = tokio::spawn(serve_http(listener, operations_router, shutdown_rx));

    tracing::info!(%operations_address, "worker service ready");
    shutdown_signal().await;
    health.set_initialized(false).await;
    let _ = shutdown_tx.send(true);
    operations_task
        .await
        .context("join operations listener")??;
    probe_task.await.context("join RabbitMQ probe")?;
    cleanup_task.await.context("join file cleanup")?;
    database.close().await;
    Ok(())
}

async fn file_cleanup_loop(
    files: PostgresFileStore,
    storage: S3FileStorage,
    mut shutdown: watch::Receiver<bool>,
) {
    let mut interval = tokio::time::interval(CLEANUP_INTERVAL);
    loop {
        tokio::select! {
            _ = interval.tick() => {
                match cleanup_expired(&files, &storage).await {
                    Ok(count) if count > 0 => tracing::info!(count, "expired file uploads removed"),
                    Ok(_) => {},
                    Err(error) => tracing::warn!(%error, "file cleanup failed"),
                }
            }
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() { break; }
            }
        }
    }
}
