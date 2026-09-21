use super::RoomOperationError;
use crate::ports::{RoomStore, StoreError};
use mahfel_domain::{ActorId, Room, RoomId, RoomMember, RoomRole, RoomStatus, RoomVisibility};
use time::OffsetDateTime;

/// Join a public room, or return the room when the actor is already a member.
///
/// Idempotent: joining twice returns the same room. Invite-only rooms are
/// reachable only through a manager adding the member, so joining one is a
/// `NotFound`. An archived room is closed to new members.
pub async fn join_room<R>(rooms: &R, id: RoomId, actor: ActorId) -> Result<Room, RoomOperationError>
where
    R: RoomStore + ?Sized,
{
    let room = rooms.get(id).await?.ok_or(RoomOperationError::NotFound)?;
    if rooms.get_member(id, actor).await?.is_some() {
        return Ok(room);
    }
    if room.status == RoomStatus::Archived {
        return Err(RoomOperationError::Archived);
    }
    if room.visibility != RoomVisibility::Public {
        return Err(RoomOperationError::NotFound);
    }
    let member = RoomMember::new(id, actor, RoomRole::Member, OffsetDateTime::now_utc());
    match rooms.add_member(&member).await {
        Ok(()) => Ok(room),
        // A concurrent join already created the membership: still idempotent.
        Err(StoreError::Conflict(_)) => Ok(room),
        Err(error) => Err(error.into()),
    }
}
