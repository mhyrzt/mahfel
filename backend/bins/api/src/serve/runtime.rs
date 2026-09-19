use super::{
    coordination::{self, Coordination},
    database::{self, Database},
    telemetry::Telemetry,
};
use anyhow::{Context, Result};
use axum::Router;
use mahfel_config::Settings;
use mahfel_domain::NodeId;
use mahfel_telemetry::{HealthState, serve_http, shutdown_signal};
use std::net::SocketAddr;
use tokio::{net::TcpListener, sync::watch, task::JoinHandle};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

/// The running service: listeners, probe, and the shutdown handshake.
pub(super) struct Runtime {
    node_id: NodeId,
    app_address: SocketAddr,
    operations_address: SocketAddr,
    health: HealthState,
    pool: sqlx::PgPool,
    shutdown_tx: watch::Sender<bool>,
    app_task: JoinHandle<std::io::Result<()>>,
    operations_task: JoinHandle<std::io::Result<()>>,
    probe_task: JoinHandle<()>,
    redis_probe_task: JoinHandle<()>,
}

/// Bind listeners, start the probe, and hand back a runnable service.
pub(super) async fn start(
    settings: &Settings,
    node_id: NodeId,
    app: Router,
    telemetry: &Telemetry,
    database: &Database,
    coordination: &Coordination,
    shutdown: (watch::Sender<bool>, watch::Receiver<bool>),
) -> Result<Runtime> {
    let (shutdown_tx, shutdown_rx) = shutdown;
    let app_address = settings.api.http_bind.parse::<SocketAddr>()?;
    let operations_address = settings.api.operations_bind.parse::<SocketAddr>()?;

    let app_task = spawn_listener(app_address, app, shutdown_rx.clone()).await?;
    // The interactive docs UI stays on the private operations listener
    // (ADR 0011, ADR 0015); the machine-readable document is served publicly.
    let operations = telemetry.operations.clone().merge(Scalar::with_url(
        "/docs",
        crate::http::routes::ApiDoc::openapi(),
    ));
    let operations_task =
        spawn_listener(operations_address, operations, shutdown_rx.clone()).await?;
    let probe_task = database::spawn_probe(database, telemetry, shutdown_rx.clone());
    let redis_probe_task = coordination::spawn_probe(coordination, telemetry, shutdown_rx);

    Ok(Runtime {
        node_id,
        app_address,
        operations_address,
        health: telemetry.health.clone(),
        pool: database.pool.clone(),
        shutdown_tx,
        app_task,
        operations_task,
        probe_task,
        redis_probe_task,
    })
}

impl Runtime {
    /// Announce readiness, wait for the shutdown signal, then drain.
    pub(super) async fn run(self) -> Result<()> {
        tracing::info!(
            app_address = %self.app_address,
            operations_address = %self.operations_address,
            node_id = %self.node_id,
            "API service ready"
        );

        shutdown_signal().await;
        self.shutdown().await
    }

    async fn shutdown(self) -> Result<()> {
        self.health.set_initialized(false).await;
        let _ = self.shutdown_tx.send(true);
        self.app_task.await.context("join application listener")??;
        self.operations_task
            .await
            .context("join operations listener")??;
        self.probe_task.await.context("join PostgreSQL probe")?;
        self.redis_probe_task.await.context("join Redis probe")?;
        self.pool.close().await;
        Ok(())
    }
}

async fn spawn_listener(
    address: SocketAddr,
    router: Router,
    shutdown: watch::Receiver<bool>,
) -> Result<JoinHandle<std::io::Result<()>>> {
    let listener = TcpListener::bind(address).await?;
    Ok(tokio::spawn(serve_http(listener, router, shutdown)))
}
