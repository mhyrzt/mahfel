//! Room use-case failures.
//!
//! One error type covers every room operation. The variant set is the union of
//! what the operations can return; a specific operation documents which
//! variants it produces. Keeping one type means the `ApplicationError`
//! mapping and the transport conversion exist once rather than per operation.

use crate::error::{ApplicationError, ErrorKind, GuardError};
use crate::ports::StoreError;
use mahfel_domain::{MembershipError, RoomError as DomainError, ValidationError};
use thiserror::Error;

/// Why a room operation failed.
#[derive(Debug, Error)]
pub enum RoomOperationError {
    /// The room does not exist. `get_room` and `join_room` also use this for a
    /// caller who is not permitted to see the room, so existence cannot be
    /// probed.
    #[error("room not found")]
    NotFound,
    /// The caller is authenticated but lacks the required room role.
    #[error("permission denied")]
    Forbidden,
    /// The room exists but is archived; it is closed to new members.
    #[error("room is archived")]
    Archived,
    #[error("invalid room title: {0}")]
    InvalidTitle(#[from] ValidationError),
    #[error("invalid room state: {0}")]
    Domain(#[from] DomainError),
    #[error("the actor is already a member")]
    AlreadyMember,
    /// The referenced actor has no row, so it cannot be granted membership.
    #[error("actor not found")]
    ActorNotFound,
    #[error("the last manager cannot be removed")]
    LastManager,
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

impl ApplicationError for RoomOperationError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::NotFound | Self::ActorNotFound => ErrorKind::NotFound,
            Self::Forbidden => ErrorKind::Forbidden,
            Self::Archived | Self::Domain(_) | Self::AlreadyMember | Self::LastManager => {
                ErrorKind::Conflict
            }
            Self::InvalidTitle(_) => ErrorKind::Unprocessable,
            Self::Store(_) => ErrorKind::Internal,
        }
    }
}

impl From<GuardError> for RoomOperationError {
    fn from(error: GuardError) -> Self {
        match error {
            GuardError::NotFound => Self::NotFound,
            GuardError::Forbidden => Self::Forbidden,
            GuardError::Store(error) => Self::Store(error),
        }
    }
}

impl From<MembershipError> for RoomOperationError {
    fn from(error: MembershipError) -> Self {
        match error {
            MembershipError::NotFound => Self::NotFound,
            MembershipError::LastManager => Self::LastManager,
        }
    }
}
