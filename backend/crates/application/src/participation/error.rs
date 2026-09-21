//! Participation use-case failures.
//!
//! One error type covers opening, closing, querying, and correcting
//! participation records. Keeping one type means the `ApplicationError`
//! mapping and the transport conversion exist once rather than per operation.

use crate::error::{ApplicationError, ErrorKind, GuardError};
use crate::ports::StoreError;
use mahfel_domain::IntervalError;
use thiserror::Error;

/// Why a participation operation failed.
#[derive(Debug, Error)]
pub enum ParticipationError {
    /// The participant, session, or interval does not exist, or the caller may
    /// not see it. Collapsed so existence cannot be probed.
    #[error("participation record not found")]
    NotFound,
    #[error("permission denied")]
    Forbidden,
    /// The interval is already closed, or one is already open for the
    /// participant, or it was already superseded.
    #[error("participation conflict: {0}")]
    Conflict(#[from] IntervalError),
    #[error("invalid correction: {0}")]
    InvalidCorrection(&'static str),
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

impl ApplicationError for ParticipationError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::NotFound => ErrorKind::NotFound,
            Self::Forbidden => ErrorKind::Forbidden,
            Self::Conflict(_) | Self::InvalidCorrection(_) => ErrorKind::Conflict,
            Self::Store(_) => ErrorKind::Internal,
        }
    }
}

impl From<GuardError> for ParticipationError {
    fn from(error: GuardError) -> Self {
        match error {
            GuardError::NotFound => Self::NotFound,
            GuardError::Forbidden => Self::Forbidden,
            GuardError::Store(error) => Self::Store(error),
        }
    }
}
