use anyhow::{Context, Result};
use mahfel_config::Settings;
use mahfel_messaging::connect;
use mahfel_telemetry::{HealthState, init_tracing, operations, shutdown_signal};
use std::{net::SocketAddr, time::Duration};
use tokio::sync::watch;

const SERVICE: &str = "mahfel-worker";

#[tokio::main]
async fn main() -> Result<()> {
    let settings = Settings::load().context("load configuration")?;
    init_tracing(&settings.telemetry.log_filter).context("initialize tracing")?;
    let connection = connect(&settings.worker.amqp_url)
        .await
        .context("connect to RabbitMQ")?;
    let (operations_router, health) = operations(SERVICE, env!("CARGO_PKG_VERSION"))?;
    health.set_check("rabbitmq", true).await;
    health.set_initialized(true).await;

    let operations_address = settings.worker.operations_bind.parse::<SocketAddr>()?;
    let listener = tokio::net::TcpListener::bind(operations_address).await?;
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let probe_task = spawn_amqp_probe(connection, health.clone(), shutdown_rx.clone());
    let operations_task = tokio::spawn(async move {
        axum::serve(listener, operations_router)
            .with_graceful_shutdown(wait_for_shutdown(shutdown_rx))
            .await
    });

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

fn spawn_amqp_probe(
    connection: lapin::Connection,
    health: HealthState,
    mut shutdown: watch::Receiver<bool>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(5)) => {
                    health.set_check("rabbitmq", connection.status().connected()).await;
                }
                changed = shutdown.changed() => {
                    if changed.is_err() || *shutdown.borrow() {
                        break;
                    }
                }
            }
        }
    })
}

async fn wait_for_shutdown(mut shutdown: watch::Receiver<bool>) {
    while !*shutdown.borrow() && shutdown.changed().await.is_ok() {}
}
