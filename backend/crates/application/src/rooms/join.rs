use crate::error::{ApplicationError, ErrorKind};
use crate::ports::{RoomStore, StoreError};
use mahfel_domain::{ActorId, Room, RoomId, RoomMember, RoomRole, RoomStatus, RoomVisibility};
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Debug, Error)]
pub enum JoinRoomError {
    /// Covers both a missing room and an invite-only room, so a caller cannot
    /// distinguish "no such room" from "not allowed to join".
    #[error("room not found")]
    NotFound,
    /// The room exists but is archived; it is closed to new members.
    #[error("room is archived")]
    Archived,
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

impl ApplicationError for JoinRoomError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::NotFound => ErrorKind::NotFound,
            Self::Archived => ErrorKind::Conflict,
            Self::Store(_) => ErrorKind::Internal,
        }
    }
}

/// Join a public room, or return the room when the actor is already a member.
///
/// Idempotent: joining twice returns the same room. Invite-only rooms are
/// reachable only through a manager adding the member, so joining one is a
/// `NotFound`. An archived room is closed to new members.
pub async fn join_room<R>(rooms: &R, id: RoomId, actor: ActorId) -> Result<Room, JoinRoomError>
where
    R: RoomStore,
{
    let room = rooms.get(id).await?.ok_or(JoinRoomError::NotFound)?;
    if rooms.get_member(id, actor).await?.is_some() {
        return Ok(room);
    }
    if room.status == RoomStatus::Archived {
        return Err(JoinRoomError::Archived);
    }
    if room.visibility != RoomVisibility::Public {
        return Err(JoinRoomError::NotFound);
    }
    let member = RoomMember::new(id, actor, RoomRole::Member, OffsetDateTime::now_utc());
    match rooms.add_member(&member).await {
        Ok(()) => Ok(room),
        // A concurrent join already created the membership: still idempotent.
        Err(StoreError::Conflict(_)) => Ok(room),
        Err(error) => Err(error.into()),
    }
}
