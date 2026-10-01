//! Session use-case failures.
//!
//! One error type covers every session operation, including participation. The
//! variant set is the union of what the operations can return; a specific
//! operation documents which variants it produces. Keeping one type means the
//! `ApplicationError` mapping and the transport conversion exist once rather
//! than per operation.

use crate::error::{ApplicationError, ErrorKind, GuardError};
use crate::ports::StoreError;
use mahfel_domain::{
    IntervalError, ParticipantError, SessionError as DomainError, ValidationError,
};
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
    #[error("recording capture is unavailable; the session remains scheduled")]
    RecordingUnavailable,
    #[error("session is not live")]
    NotLive,
    #[error("invalid session transition: {0}")]
    Domain(#[from] DomainError),
    #[error("invalid session details: {0}")]
    Validation(#[from] ValidationError),
    #[error("invalid participant transition: {0}")]
    Participant(#[from] ParticipantError),
    /// Participation-record failure while joining or leaving. The participant
    /// operation itself succeeded or failed on its own terms; this reports a
    /// durable recording problem, which is an internal failure, not a denial.
    #[error("participation record failure: {0}")]
    Participation(#[from] IntervalError),
    /// Admission failure while joining. Distinct from a denial: the caller was
    /// authorized, but the lobby could not queue them.
    #[error("admission failure: {0}")]
    Admission(#[from] crate::admission::AdmissionOperationError),
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
            Self::RecordingUnavailable => ErrorKind::Unavailable,
            Self::Validation(_) => ErrorKind::Unprocessable,
            Self::Participation(_) => ErrorKind::Internal,
            Self::Admission(error) => error.kind(),
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
