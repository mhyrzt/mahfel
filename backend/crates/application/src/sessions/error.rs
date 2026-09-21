//! Session use-case failures.
//!
//! One error type covers every session operation, including participation. The
//! variant set is the union of what the operations can return; a specific
//! operation documents which variants it produces. Keeping one type means the
//! `ApplicationError` mapping and the transport conversion exist once rather
//! than per operation.

use crate::error::{ApplicationError, ErrorKind, GuardError};
use crate::ports::StoreError;
use mahfel_domain::{ParticipantError, SessionError as DomainError};
use thiserror::Error;

/// Why a session operation failed.
#[derive(Debug, Error)]
pub enum SessionOperationError {
    /// The session, room, or participant does not exist. Join and leave also
    /// use this for a caller who is not permitted to see it, so existence
    /// cannot be probed.
    #[error("session not found")]
    NotFound,
    #[error("permission denied")]
    Forbidden,
    /// Starting a session when the room already has a live one.
    #[error("a live session already exists for this room")]
    Conflict,
    #[error("session is not live")]
    NotLive,
    #[error("invalid session transition: {0}")]
    Domain(#[from] DomainError),
    #[error("invalid participant transition: {0}")]
    Participant(#[from] ParticipantError),
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

impl ApplicationError for SessionOperationError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::NotFound => ErrorKind::NotFound,
            Self::Forbidden => ErrorKind::Forbidden,
            Self::Conflict | Self::NotLive | Self::Domain(_) | Self::Participant(_) => {
                ErrorKind::Conflict
            }
            Self::Store(_) => ErrorKind::Internal,
        }
    }
}

impl From<GuardError> for SessionOperationError {
    fn from(error: GuardError) -> Self {
        match error {
            GuardError::NotFound => Self::NotFound,
            GuardError::Forbidden => Self::Forbidden,
            GuardError::Store(error) => Self::Store(error),
        }
    }
}
