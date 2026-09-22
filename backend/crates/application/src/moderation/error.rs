//! Moderation use-case failures.

use crate::error::{ApplicationError, ErrorKind, GuardError};
use crate::ports::{MediaGatewayError, StoreError};
use mahfel_domain::{BlockError, ParticipantError, ValidationError};
use thiserror::Error;

/// Why a moderation action failed.
#[derive(Debug, Error)]
pub enum ModerationError {
    /// The session, room, target participant, or block does not exist, or the
    /// caller may not learn that it does.
    ///
    /// A target participant that is absent or no longer active lands here,
    /// which is why identity is resolved against the authoritative store before
    /// any media cleanup: the media plane holds no participant registry and
    /// must not gain one.
    #[error("not found")]
    NotFound,
    /// The caller does not hold the capability this action requires.
    #[error("permission denied")]
    Forbidden,
    /// The target is not muted, so there is nothing to clear.
    #[error("the participant is not muted")]
    NotMuted,
    /// A permanent mute is not the target's to clear.
    #[error("a permanent mute can only be cleared by a moderator")]
    PermanentMuteNeedsModerator,
    /// The principal is already blocked, so there is nothing to extend.
    #[error("the principal is already blocked")]
    AlreadyBlocked,
    /// The block was already lifted.
    #[error("the block is already lifted")]
    AlreadyLifted,
    #[error("invalid block state: {0}")]
    Block(#[from] BlockError),
    /// A participant-state failure moderation has no separate word for. The
    /// ones it does have are mapped by the `From` impl below.
    #[error("invalid participant state: {0}")]
    Participant(ParticipantError),
    #[error("invalid value: {0}")]
    Validation(#[from] ValidationError),
    /// The durable state changed but enforcement did not. Distinct from a
    /// storage failure because the mute or removal is already recorded and only
    /// the media effect needs reconciling.
    #[error("media enforcement failed: {0}")]
    Enforcement(#[from] MediaGatewayError),
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

impl ApplicationError for ModerationError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::NotFound => ErrorKind::NotFound,
            Self::Forbidden | Self::PermanentMuteNeedsModerator => ErrorKind::Forbidden,
            Self::NotMuted | Self::AlreadyBlocked | Self::AlreadyLifted => ErrorKind::Conflict,
            Self::Block(_) | Self::Participant(_) | Self::Validation(_) => ErrorKind::Unprocessable,
            Self::Enforcement(_) | Self::Store(_) => ErrorKind::Internal,
        }
    }
}

impl From<ParticipantError> for ModerationError {
    /// Map the participant failures that moderation has its own vocabulary for
    /// onto it, so a caller sees one variant rather than a nested one.
    ///
    /// The remaining variants keep their participant context, because they
    /// describe a state moderation has no separate word for.
    fn from(error: ParticipantError) -> Self {
        match error {
            ParticipantError::PermanentMuteNeedsModerator => Self::PermanentMuteNeedsModerator,
            other => Self::Participant(other),
        }
    }
}

impl From<GuardError> for ModerationError {
    fn from(error: GuardError) -> Self {
        match error {
            GuardError::NotFound => Self::NotFound,
            GuardError::Forbidden => Self::Forbidden,
            GuardError::Store(error) => Self::Store(error),
        }
    }
}

impl From<crate::ports::AuthorizationError> for ModerationError {
    fn from(error: crate::ports::AuthorizationError) -> Self {
        match error {
            crate::ports::AuthorizationError::Denied => Self::Forbidden,
            // A denied caller with no identity is still a caller who lacks the
            // capability, and moderation is never reachable anonymously.
            crate::ports::AuthorizationError::Unauthenticated => Self::Forbidden,
        }
    }
}
