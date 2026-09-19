//! Operations listener and metric handles created once at startup.

use crate::http::ws::WsMetrics;
use anyhow::{Context, Result};
use axum::Router;
use mahfel_postgres::PoolMetrics;
use mahfel_telemetry::{HealthState, operations};

/// Operations listener state and metric handles created once at startup.
pub(super) struct Telemetry {
    pub(super) operations: Router,
    pub(super) health: HealthState,
    pub(super) pool_metrics: PoolMetrics,
    pub(super) ws_metrics: WsMetrics,
}

impl Telemetry {
    pub(super) fn start(service: &str, version: &str) -> Result<Self> {
        let (operations, health) = operations(service, version)?;
        let pool_metrics = PoolMetrics::register(&health.registry())?;
        let ws_metrics =
            WsMetrics::register(&health.registry()).context("register websocket metrics")?;
        Ok(Self {
            operations,
            health,
            pool_metrics,
            ws_metrics,
        })
    }
}
