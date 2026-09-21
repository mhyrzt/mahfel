//! Ephemeral coordination port (ADR 0006).
//!
//! Redis holds replaceable coordination state only. Every write carries a TTL,
//! and an outage must surface as [`CoordinationError::Unavailable`] so callers
//! degrade instead of treating the request as malformed.

use super::BoxFuture;
use crate::error::{ApplicationError, ErrorKind};
use std::time::Duration;
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
/// consumer needs a substitution seam. Methods return [`BoxFuture`] so the port
/// is dyn-compatible.
pub trait CoordinationStore: Send + Sync {
    /// Store a value that expires after `ttl`.
    fn set_with_ttl<'a>(
        &'a self,
        key: &'a str,
        value: &'a [u8],
        ttl: Duration,
    ) -> BoxFuture<'a, Result<(), CoordinationError>>;

    /// Read a value, if present and unexpired.
    fn get<'a>(&'a self, key: &'a str)
    -> BoxFuture<'a, Result<Option<Vec<u8>>, CoordinationError>>;

    /// Remove a value.
    fn delete<'a>(&'a self, key: &'a str) -> BoxFuture<'a, Result<(), CoordinationError>>;

    /// Publish a payload to a fan-out channel.
    fn publish<'a>(
        &'a self,
        channel: &'a str,
        payload: &'a [u8],
    ) -> BoxFuture<'a, Result<(), CoordinationError>>;
}
