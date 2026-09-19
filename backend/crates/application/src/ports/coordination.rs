//! Ephemeral coordination port (ADR 0006).
//!
//! Redis holds replaceable coordination state only. Every write carries a TTL,
//! and an outage must surface as [`CoordinationError::Unavailable`] so callers
//! degrade instead of treating the request as malformed.

use crate::error::{ApplicationError, ErrorKind};
use std::{future::Future, time::Duration};
use thiserror::Error;

/// Ephemeral coordination failure.
#[derive(Debug, Error)]
pub enum CoordinationError {
    /// The backend is unreachable or timed out. Callers should degrade.
    #[error("coordination unavailable: {0}")]
    Unavailable(String),
    /// The backend responded, but the operation failed. Messages must not
    /// reach clients.
    #[error("coordination failure: {0}")]
    Unexpected(String),
}

impl ApplicationError for CoordinationError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Unavailable(_) => ErrorKind::Unavailable,
            Self::Unexpected(_) => ErrorKind::Internal,
        }
    }
}

/// Minimal ephemeral coordination primitives.
///
/// Keys are namespaced by the adapter; values are opaque bytes. The port stays
/// deliberately small — subscribe stream semantics are adapter-specific until a
/// consumer needs a substitution seam.
pub trait CoordinationStore: Send + Sync {
    /// Store a value that expires after `ttl`.
    fn set_with_ttl(
        &self,
        key: &str,
        value: &[u8],
        ttl: Duration,
    ) -> impl Future<Output = Result<(), CoordinationError>> + Send;

    /// Read a value, if present and unexpired.
    fn get(
        &self,
        key: &str,
    ) -> impl Future<Output = Result<Option<Vec<u8>>, CoordinationError>> + Send;

    /// Remove a value.
    fn delete(&self, key: &str) -> impl Future<Output = Result<(), CoordinationError>> + Send;

    /// Publish a payload to a fan-out channel.
    fn publish(
        &self,
        channel: &str,
        payload: &[u8],
    ) -> impl Future<Output = Result<(), CoordinationError>> + Send;
}
