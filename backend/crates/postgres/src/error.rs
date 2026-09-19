use thiserror::Error;

/// PostgreSQL adapter failure.
///
/// Infrastructure errors are converted into this type so callers never depend
/// on `sqlx`, `prometheus`, or stored-value details directly.
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
    #[error("stored domain value is invalid: {0}")]
    InvalidDomainValue(#[from] mahfel_domain::InvalidStatus),
    /// A stored text value no longer satisfies its domain invariant.
    #[error("stored text value is invalid: {0}")]
    InvalidStoredValue(&'static str),
}
