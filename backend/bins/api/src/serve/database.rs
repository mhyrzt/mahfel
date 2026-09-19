use super::telemetry::Telemetry;
use crate::settings;
use anyhow::{Context, Result};
use mahfel_config::Settings;
use mahfel_postgres::{connect, migrate, ping};
use std::time::Duration;
use tokio::{sync::watch, task::JoinHandle};

const POSTGRES_PROBE: &str = "postgresql";
const PROBE_INTERVAL: Duration = Duration::from_secs(5);

/// Pooled database connection. Closing a clone closes the shared pool.
pub(super) struct Database {
    pub(super) pool: sqlx::PgPool,
}

impl Database {
    /// Connect, optionally migrate, and publish the first readiness result.
    pub(super) async fn connect(settings: &Settings, telemetry: &Telemetry) -> Result<Self> {
        let pool = connect(&settings::connection(settings), settings::pool(settings))
            .await
            .context("connect to PostgreSQL")?;
        if settings.database.migrate_on_start {
            migrate(&pool)
                .await
                .context("run startup PostgreSQL migrations")?;
        }
        telemetry.pool_metrics.observe(&pool);
        telemetry
            .health
            .set_check(POSTGRES_PROBE, ping(&pool).await.is_ok())
            .await;
        telemetry.health.set_initialized(true).await;
        Ok(Self { pool })
    }
}

/// Spawn the periodic readiness probe; the runtime drains the handle.
pub(super) fn spawn_probe(
    database: &Database,
    telemetry: &Telemetry,
    shutdown: watch::Receiver<bool>,
) -> JoinHandle<()> {
    let pool = database.pool.clone();
    let metrics = telemetry.pool_metrics.clone();
    let health = telemetry.health.clone();
    mahfel_telemetry::spawn_probe(
        POSTGRES_PROBE,
        PROBE_INTERVAL,
        health,
        shutdown,
        move || {
            let pool = pool.clone();
            let metrics = metrics.clone();
            async move {
                let healthy = ping(&pool).await.is_ok();
                metrics.observe(&pool);
                if !healthy {
                    metrics.record_probe_failure();
                }
                healthy
            }
        },
    )
}
