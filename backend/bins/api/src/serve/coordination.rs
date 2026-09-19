//! Ephemeral coordination backend (ADR 0006).

use super::telemetry::Telemetry;
use crate::settings;
use anyhow::{Context, Result};
use mahfel_config::Settings;
use mahfel_redis::RedisCoordination;
use std::time::Duration;
use tokio::{sync::watch, task::JoinHandle};

const REDIS_PROBE: &str = "redis";
const PROBE_INTERVAL: Duration = Duration::from_secs(5);

/// Connected coordination client, held for the consumers that arrive later
/// (cross-replica fanout, media-node registry, presence).
pub(super) struct Coordination {
    pub(super) store: RedisCoordination,
}

impl Coordination {
    pub(super) async fn connect(settings: &Settings, telemetry: &Telemetry) -> Result<Self> {
        let store =
            RedisCoordination::connect(&settings::redis(settings), telemetry.redis_metrics.clone())
                .await
                .context("connect to Redis")?;
        telemetry
            .health
            .set_check(REDIS_PROBE, store.ping().await.is_ok())
            .await;
        Ok(Self { store })
    }
}

/// Spawn the periodic readiness probe; the runtime drains the handle.
pub(super) fn spawn_probe(
    coordination: &Coordination,
    telemetry: &Telemetry,
    shutdown: watch::Receiver<bool>,
) -> JoinHandle<()> {
    let store = coordination.store.clone();
    let metrics = telemetry.redis_metrics.clone();
    let health = telemetry.health.clone();
    mahfel_telemetry::spawn_probe(REDIS_PROBE, PROBE_INTERVAL, health, shutdown, move || {
        let store = store.clone();
        let metrics = metrics.clone();
        async move {
            let healthy = store.ping().await.is_ok();
            if !healthy {
                metrics.probe_failure();
            }
            healthy
        }
    })
}
