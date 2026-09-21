use super::ManagerGuardError;
use crate::error::{ApplicationError, ErrorKind};
use crate::ports::{RoomStore, StoreError};
use mahfel_domain::{
    ActorId, MembershipError, RoomId, RoomMember, RoomRole, ensure_member_removable,
};
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Debug, Error)]
pub enum AddMemberError {
    #[error("room not found")]
    NotFound,
    #[error("permission denied")]
    Forbidden,
    #[error("user is already a member")]
    AlreadyMember,
    #[error("user not found")]
    UserNotFound,
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

impl From<ManagerGuardError> for AddMemberError {
    fn from(error: ManagerGuardError) -> Self {
        match error {
            ManagerGuardError::NotFound => Self::NotFound,
            ManagerGuardError::Forbidden => Self::Forbidden,
            ManagerGuardError::Store(error) => Self::Store(error),
        }
    }
}

impl ApplicationError for AddMemberError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::NotFound => ErrorKind::NotFound,
            Self::Forbidden => ErrorKind::Forbidden,
            Self::AlreadyMember => ErrorKind::Conflict,
            Self::UserNotFound => ErrorKind::NotFound,
            Self::Store(_) => ErrorKind::Internal,
        }
    }
}

/// Manager-only: add a member, granting access immediately (auto join).
pub async fn add_member<R>(
    rooms: &R,
    room_id: RoomId,
    actor: ActorId,
    target: ActorId,
    role: RoomRole,
) -> Result<RoomMember, AddMemberError>
where
    R: RoomStore,
{
    super::require_manager(rooms, room_id, actor).await?;
    let member = RoomMember::new(room_id, target, role, OffsetDateTime::now_utc());
    match rooms.add_member(&member).await {
        Ok(()) => Ok(member),
        Err(StoreError::Conflict(_)) => Err(AddMemberError::AlreadyMember),
        // The target user has no row: a client error, not an internal failure.
        Err(StoreError::MissingReference(_)) => Err(AddMemberError::UserNotFound),
        Err(error) => Err(error.into()),
    }
}

#[derive(Debug, Error)]
pub enum RemoveMemberError {
    #[error("room not found")]
    NotFound,
    #[error("permission denied")]
    Forbidden,
    #[error("the last manager cannot be removed")]
    LastManager,
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

impl From<ManagerGuardError> for RemoveMemberError {
    fn from(error: ManagerGuardError) -> Self {
        match error {
            ManagerGuardError::NotFound => Self::NotFound,
            ManagerGuardError::Forbidden => Self::Forbidden,
            ManagerGuardError::Store(error) => Self::Store(error),
        }
    }
}

impl ApplicationError for RemoveMemberError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::NotFound => ErrorKind::NotFound,
            Self::Forbidden => ErrorKind::Forbidden,
            Self::LastManager => ErrorKind::Conflict,
            Self::Store(_) => ErrorKind::Internal,
        }
    }
}

/// Manager-only: remove a membership, removing access. The last manager may
/// not be removed.
pub async fn remove_member<R>(
    rooms: &R,
    room_id: RoomId,
    actor: ActorId,
    target: ActorId,
) -> Result<(), RemoveMemberError>
where
    R: RoomStore,
{
    super::require_manager(rooms, room_id, actor).await?;
    let members = rooms.list_members(room_id).await?;
    match ensure_member_removable(&members, target) {
        Ok(_) => {}
        Err(MembershipError::NotFound) => return Err(RemoveMemberError::NotFound),
        Err(MembershipError::LastManager) => return Err(RemoveMemberError::LastManager),
    }
    if !rooms.remove_member(room_id, target).await? {
        return Err(RemoveMemberError::NotFound);
    }
    Ok(())
}
