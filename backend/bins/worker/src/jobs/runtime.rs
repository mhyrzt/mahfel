use anyhow::{Context, Result};
use mahfel_config::WorkerSettings;
use mahfel_messaging::{ConfirmedPublisher, JobConsumer, connect, declare_topology};
use mahfel_postgres::{JobStore, PostgresFileStore};
use mahfel_storage::S3FileStorage;
use mahfel_telemetry::HealthState;
use std::time::Duration;
use tokio::sync::watch;

use super::{JobMetrics, RECONNECT_DELAY, consumer::consume_loop, relay::relay_loop};

pub async fn run(
    settings: WorkerSettings,
    store: JobStore,
    files: PostgresFileStore,
    storage: S3FileStorage,
    health: HealthState,
    metrics: JobMetrics,
    mut shutdown: watch::Receiver<bool>,
) {
    loop {
        if *shutdown.borrow() {
            break;
        }
        let result = run_connected(
            &settings,
            &store,
            &files,
            &storage,
            &health,
            &metrics,
            shutdown.clone(),
        )
        .await;
        health.set_check("rabbitmq", false).await;
        if *shutdown.borrow() {
            break;
        }
        if let Err(error) = result {
            tracing::warn!(%error, "job transport disconnected; reconnecting");
        }
        while store.ping().await.is_err() {
            health.set_check("database", false).await;
            tokio::select! {
                _ = tokio::time::sleep(RECONNECT_DELAY) => {},
                _ = shutdown.changed() => return,
            }
        }
        health.set_check("database", true).await;
        tokio::select! {
            _ = tokio::time::sleep(RECONNECT_DELAY) => {},
            _ = shutdown.changed() => break,
        }
        // The readiness probe tracks the next successful connection below.
        health.set_check("rabbitmq", false).await;
    }
}
async fn run_connected(
    settings: &WorkerSettings,
    store: &JobStore,
    files: &PostgresFileStore,
    storage: &S3FileStorage,
    health: &HealthState,
    metrics: &JobMetrics,
    mut shutdown: watch::Receiver<bool>,
) -> Result<()> {
    let connection = connect(&settings.amqp_url)
        .await
        .context("connect to RabbitMQ")?;
    declare_topology(&connection)
        .await
        .context("declare job topology")?;
    let publisher = ConfirmedPublisher::new(&connection).await?;
    let consumer = JobConsumer::new(&connection, settings.prefetch).await?;
    health.set_check("rabbitmq", true).await;
    let relay_store = store.clone();
    let relay_metrics = metrics.clone();
    let mut relay =
        tokio::spawn(async move { relay_loop(relay_store, publisher, relay_metrics).await });
    let consume_store = store.clone();
    let consume_files = files.clone();
    let consume_storage = storage.clone();
    let consume_metrics = metrics.clone();
    let consumer_settings = settings.clone();
    let drain_timeout = Duration::from_secs(settings.drain_timeout_seconds);
    let intake_shutdown = shutdown.clone();
    let mut intake = tokio::spawn(async move {
        consume_loop(
            consumer,
            consume_store,
            consume_files,
            consume_storage,
            consume_metrics,
            intake_shutdown,
            consumer_settings,
        )
        .await
    });
    // The outer run loop will reconnect both channels if either half fails.
    enum Exit {
        Shutdown,
        Relay(Result<()>),
        Consumer(Result<()>),
    }
    let exit = tokio::select! {
        result = &mut relay => Exit::Relay(result.context("relay task")?),
        result = &mut intake => Exit::Consumer(result.context("consumer task")?),
        _ = shutdown.changed() => Exit::Shutdown,
    };
    let result = match exit {
        Exit::Shutdown => {
            relay.abort();
            let _ = relay.await;
            match tokio::time::timeout(drain_timeout + Duration::from_secs(2), &mut intake).await {
                Ok(result) => result.context("consumer task")?,
                Err(_) => {
                    intake.abort();
                    let _ = intake.await;
                    Ok(())
                }
            }
        }
        Exit::Relay(result) => {
            intake.abort();
            let _ = intake.await;
            result
        }
        Exit::Consumer(result) => {
            relay.abort();
            let _ = relay.await;
            result
        }
    };
    let _ = connection.close(200, "worker reconnect".into()).await;
    result
}
