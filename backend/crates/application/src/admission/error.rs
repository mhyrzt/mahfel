//! Admission use-case failures.

use crate::error::{ApplicationError, ErrorKind, GuardError};
use crate::ports::StoreError;
use mahfel_domain::{AdmissionError, RoomError, SessionError, ValidationError};
use thiserror::Error;

/// Why an admission operation failed.
#[derive(Debug, Error)]
pub enum AdmissionOperationError {
    /// The session, room, or request does not exist, or the caller may not learn
    /// that it does.
    #[error("not found")]
    NotFound,
    #[error("permission denied")]
    Forbidden,
    /// The request was already decided, so the second decision is refused rather
    /// than overwriting the first.
    #[error("admission request was already decided")]
    AlreadyDecided,
    /// A principal would be admitted but is not actually waiting.
    #[error("the principal is not waiting in this lobby")]
    NotWaiting,
    /// The lobby is at capacity, so a new request cannot be queued.
    #[error("lobby is at capacity")]
    LobbyFull,
    #[error("invalid session state: {0}")]
    Domain(#[from] SessionError),
    #[error("invalid room state: {0}")]
    Room(#[from] RoomError),
    #[error("invalid admission state: {0}")]
    Admission(#[from] AdmissionError),
    #[error("invalid value: {0}")]
    Validation(#[from] ValidationError),
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

impl ApplicationError for AdmissionOperationError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::NotFound => ErrorKind::NotFound,
            Self::Forbidden => ErrorKind::Forbidden,
            Self::AlreadyDecided | Self::NotWaiting | Self::LobbyFull => ErrorKind::Conflict,
            Self::Domain(_) | Self::Room(_) | Self::Admission(_) | Self::Validation(_) => {
                ErrorKind::Unprocessable
            }
            Self::Store(_) => ErrorKind::Internal,
        }
    }
}

impl From<GuardError> for AdmissionOperationError {
    fn from(error: GuardError) -> Self {
        match error {
            GuardError::NotFound => Self::NotFound,
            GuardError::Forbidden => Self::Forbidden,
            GuardError::Store(error) => Self::Store(error),
        }
    }
}
