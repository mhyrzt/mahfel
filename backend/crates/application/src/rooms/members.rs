use super::RoomOperationError;
use crate::ports::{RoomStore, StoreError};
use mahfel_domain::{ActorId, RoomId, RoomMember, RoomRole, ensure_member_removable};
use time::OffsetDateTime;

/// Manager-only: add a member, granting access immediately (auto join).
pub async fn add_member<R>(
    rooms: &R,
    room_id: RoomId,
    actor: ActorId,
    target: ActorId,
    role: RoomRole,
) -> Result<RoomMember, RoomOperationError>
where
    R: RoomStore + ?Sized,
{
    super::require_manager(rooms, room_id, actor).await?;
    let member = RoomMember::new(room_id, target, role, OffsetDateTime::now_utc());
    match rooms.add_member(&member).await {
        Ok(()) => Ok(member),
        Err(StoreError::Conflict(_)) => Err(RoomOperationError::AlreadyMember),
        // The target actor has no row: a client error, not an internal failure.
        Err(StoreError::MissingReference(_)) => Err(RoomOperationError::ActorNotFound),
        Err(error) => Err(error.into()),
    }
}

/// Manager-only: remove a membership, removing access. The last manager may
/// not be removed.
pub async fn remove_member<R>(
    rooms: &R,
    room_id: RoomId,
    actor: ActorId,
    target: ActorId,
) -> Result<(), RoomOperationError>
where
    R: RoomStore + ?Sized,
{
    super::require_manager(rooms, room_id, actor).await?;
    let members = rooms.list_members(room_id).await?;
    ensure_member_removable(&members, target)?;
    if !rooms.remove_member(room_id, target).await? {
        return Err(RoomOperationError::NotFound);
    }
    Ok(())
}
