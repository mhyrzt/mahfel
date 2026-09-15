use prometheus::{IntCounter, IntGauge, Opts, Registry};
use sqlx::{
    migrate::Migrator,
    postgres::{PgPool, PgPoolOptions},
};
use std::time::Duration;
use thiserror::Error;

static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

#[derive(Clone, Copy, Debug)]
pub struct PoolSettings {
    pub min_connections: u32,
    pub max_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

#[derive(Clone)]
pub struct PoolMetrics {
    connections: IntGauge,
    idle_connections: IntGauge,
    probe_failures: IntCounter,
}

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("invalid PostgreSQL pool configuration: {0}")]
    InvalidConfiguration(&'static str),
    #[error("PostgreSQL operation failed: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("PostgreSQL migration failed: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("PostgreSQL metrics registration failed: {0}")]
    Metrics(#[from] prometheus::Error),
}

impl PoolSettings {
    pub fn validate(self) -> Result<Self, DatabaseError> {
        if self.max_connections == 0 {
            return Err(DatabaseError::InvalidConfiguration(
                "max_connections must be greater than zero",
            ));
        }
        if self.min_connections > self.max_connections {
            return Err(DatabaseError::InvalidConfiguration(
                "min_connections must not exceed max_connections",
            ));
        }
        if self.acquire_timeout.is_zero() {
            return Err(DatabaseError::InvalidConfiguration(
                "acquire_timeout must be greater than zero",
            ));
        }
        Ok(self)
    }
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

pub async fn connect(database_url: &str, settings: PoolSettings) -> Result<PgPool, DatabaseError> {
    let settings = settings.validate()?;
    PgPoolOptions::new()
        .min_connections(settings.min_connections)
        .max_connections(settings.max_connections)
        .acquire_timeout(settings.acquire_timeout)
        .idle_timeout(Some(settings.idle_timeout))
        .max_lifetime(Some(settings.max_lifetime))
        .connect(database_url)
        .await
        .map_err(DatabaseError::from)
}

pub async fn migrate(pool: &PgPool) -> Result<(), DatabaseError> {
    MIGRATOR.run(pool).await.map_err(DatabaseError::from)
}

pub async fn ping(pool: &PgPool) -> Result<(), DatabaseError> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(DatabaseError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_settings() -> PoolSettings {
        PoolSettings {
            min_connections: 1,
            max_connections: 4,
            acquire_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(60),
            max_lifetime: Duration::from_secs(300),
        }
    }

    #[test]
    fn rejects_an_inverted_connection_range() {
        let settings = PoolSettings {
            min_connections: 5,
            max_connections: 4,
            ..valid_settings()
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn rejects_zero_acquire_timeout() {
        let settings = PoolSettings {
            acquire_timeout: Duration::ZERO,
            ..valid_settings()
        };
        assert!(settings.validate().is_err());
    }
}
