use crate::DatabaseError;
use prometheus::{IntCounter, IntGauge, Opts, Registry};
use sqlx::PgPool;

/// PostgreSQL pool gauges and probe counters.
#[derive(Clone)]
pub struct PoolMetrics {
    connections: IntGauge,
    idle_connections: IntGauge,
    probe_failures: IntCounter,
}

impl PoolMetrics {
    pub fn register(registry: &Registry) -> Result<Self, DatabaseError> {
        let connections = IntGauge::with_opts(Opts::new(
            "database_pool_connections",
            "Current PostgreSQL pool connections",
        ))?;
        let idle_connections = IntGauge::with_opts(Opts::new(
            "database_pool_idle_connections",
            "Current idle PostgreSQL pool connections",
        ))?;
        let probe_failures = IntCounter::with_opts(Opts::new(
            "database_probe_failures_total",
            "Total failed PostgreSQL readiness probes",
        ))?;
        registry.register(Box::new(connections.clone()))?;
        registry.register(Box::new(idle_connections.clone()))?;
        registry.register(Box::new(probe_failures.clone()))?;
        Ok(Self {
            connections,
            idle_connections,
            probe_failures,
        })
    }

    pub fn observe(&self, pool: &PgPool) {
        self.connections.set(i64::from(pool.size()));
        self.idle_connections
            .set(i64::try_from(pool.num_idle()).unwrap_or(i64::MAX));
    }

    pub fn record_probe_failure(&self) {
        self.probe_failures.inc();
    }
}
