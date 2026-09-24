use thiserror::Error;

/// Generic storage failure for application ports.
///
/// Infrastructure adapters map their concrete errors (`sqlx::Error`,
/// `DatabaseError`, etc.) into this type so `mahfel-application`
/// never depends on `sqlx`/`lapin`/`tonic` directly.
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("storage failure: {0}")]
    Unexpected(String),
    #[error("entity already exists: {0}")]
    Conflict(String),
    #[error("entity not found: {0}")]
    NotFound(String),
    /// A referenced row is missing (for example a foreign key target), so the
    /// operation cannot succeed. Distinct from [`StoreError::NotFound`], which
    /// means the requested entity itself was absent.
    #[error("referenced entity not found: {0}")]
    MissingReference(String),
}

impl StoreError {
    pub fn unexpected<E: std::fmt::Display>(err: E) -> Self {
        Self::Unexpected(err.to_string())
    }
}
