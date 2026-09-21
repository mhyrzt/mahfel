//! Invite link management and redemption failures.
//!
//! One error type covers every invite operation, so the `ApplicationError`
//! mapping and the transport conversion exist once.
//!
//! Redemption-facing failures are deliberately coarse: an invalid, expired,
//! revoked, or rate-limited attempt must be indistinguishable to the caller
//! (ADR 0021).

use crate::error::{ApplicationError, ErrorKind, GuardError};
use crate::ports::{CoordinationError, StoreError};
use mahfel_domain::{InviteLinkError, ValidationError};
use thiserror::Error;

/// Why an invite link operation failed.
#[derive(Debug, Error)]
pub enum InviteOperationError {
    /// Used for a missing link and for a caller who may not learn that it
    /// exists, so existence cannot be probed.
    #[error("invite link not found")]
    NotFound,
    #[error("permission denied")]
    Forbidden,
    #[error("invite link is revoked")]
    Revoked,
    /// Bounded generation ran out of attempts: uniqueness is not behaving.
    #[error("could not generate a unique invite code")]
    CodeExhausted,
    #[error("invalid invite expiry")]
    InvalidExpiry,
    #[error("invalid link state: {0}")]
    Domain(#[from] InviteLinkError),
    #[error("invalid value: {0}")]
    Validation(#[from] ValidationError),
    #[error("rate limiter unavailable: {0}")]
    RateLimited(#[from] CoordinationError),
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

impl InviteOperationError {
    /// Whether this is the deliberate non-enumerating redemption failure.
    #[must_use]
    pub const fn is_not_redeemable(&self) -> bool {
        matches!(self, Self::NotFound)
    }
}

impl ApplicationError for InviteOperationError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::NotFound | Self::Revoked => ErrorKind::NotFound,
            Self::Forbidden => ErrorKind::Forbidden,
            Self::Domain(InviteLinkError::AlreadyRevoked) => ErrorKind::Conflict,
            Self::Domain(_) | Self::InvalidExpiry | Self::Validation(_) => ErrorKind::Unprocessable,
            Self::RateLimited(CoordinationError::Unavailable(_)) => ErrorKind::Unavailable,
            Self::RateLimited(_) | Self::CodeExhausted | Self::Store(_) => ErrorKind::Internal,
        }
    }
}

impl From<GuardError> for InviteOperationError {
    fn from(error: GuardError) -> Self {
        match error {
            GuardError::NotFound => Self::NotFound,
            GuardError::Forbidden => Self::Forbidden,
            GuardError::Store(error) => Self::Store(error),
        }
    }
}
