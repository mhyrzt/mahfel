//! Composition root: bootstrap, wire, and run the API service.
//!
//! [`run`] reads as a short sequence of named steps. Each step lives in its own
//! module and owns one concern: operations/metrics (`telemetry`),
//! readiness/probes (`database`), transport composition (`http`), and lifecycle
//! (`runtime`).

mod coordination;
mod database;
mod http;
mod media_registry;
pub(crate) mod recordings;
mod runtime;
mod telemetry;

use anyhow::{Context, Result};
use mahfel_config::Settings;
use mahfel_domain::NodeId;
use mahfel_media_client::lazy as lazy_media_client;
use tokio::sync::watch;

const SERVICE: &str = "mahfel-api";
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) async fn run(settings: Settings) -> Result<()> {
    let telemetry = telemetry::Telemetry::start(SERVICE, VERSION)?;
    // The lazy client is a guard: binding it keeps it alive for the process.
    let _media =
        lazy_media_client(settings.api.media_grpc_url.clone()).context("configure media client")?;
    let database = database::Database::connect(&settings, &telemetry).await?;
    let coordination = coordination::Coordination::connect(&settings, &telemetry).await?;
    let registry = media_registry::RegistryService::new(
        std::sync::Arc::new(coordination.store.clone()),
        &settings.api.media_registry_secret,
        telemetry.registry_metrics.clone(),
    )?;
    let node_id = NodeId::new();
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let app = http::app(
        &settings,
        &node_id,
        &database,
        &coordination,
        &telemetry,
        shutdown_rx.clone(),
    )
    .await?;

    runtime::start(
        &settings,
        node_id,
        (app, registry),
        &telemetry,
        &database,
        &coordination,
        (shutdown_tx, shutdown_rx),
    )
    .await?
    .run()
    .await
}
