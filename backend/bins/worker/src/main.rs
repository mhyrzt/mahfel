use anyhow::{Context, Result};
use mahfel_config::Settings;
use mahfel_messaging::connect;
use mahfel_telemetry::{init_tracing, operations, serve_http, shutdown_signal, spawn_probe};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{net::TcpListener, sync::watch};

const SERVICE: &str = "mahfel-worker";
const AMQP_PROBE: &str = "rabbitmq";
const PROBE_INTERVAL: Duration = Duration::from_secs(5);

#[tokio::main]
async fn main() -> Result<()> {
    let settings = Settings::load().context("load configuration")?;
    init_tracing(&settings.telemetry.log_filter).context("initialize tracing")?;
    let connection = Arc::new(
        connect(&settings.worker.amqp_url)
            .await
            .context("connect to RabbitMQ")?,
    );
    let (operations_router, health) = operations(SERVICE, env!("CARGO_PKG_VERSION"))?;
    health.set_check(AMQP_PROBE, true).await;
    health.set_initialized(true).await;

    let operations_address = settings.worker.operations_bind.parse::<SocketAddr>()?;
    let listener = TcpListener::bind(operations_address).await?;
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
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
    Ok(())
}
