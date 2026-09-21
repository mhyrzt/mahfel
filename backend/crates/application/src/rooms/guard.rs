//! Room authorization guards shared by the room use cases.

use crate::error::GuardError;
use crate::ports::RoomStore;
use mahfel_domain::{ActorId, Room, RoomId, RoomRole};

/// Load the room and require the actor to be one of its managers.
///
/// A missing room and a non-member caller both yield [`GuardError::NotFound`]
/// so a caller cannot probe room existence; a member without the manager role
/// yields [`GuardError::Forbidden`].
pub(crate) async fn require_manager<R>(
    rooms: &R,
    room_id: RoomId,
    actor: ActorId,
) -> Result<Room, GuardError>
where
    R: RoomStore + ?Sized,
{
    let room = rooms.get(room_id).await?.ok_or(GuardError::NotFound)?;
    let membership = rooms
        .get_member(room_id, actor)
        .await?
        .ok_or(GuardError::NotFound)?;
    if membership.role != RoomRole::Manager {
        return Err(GuardError::Forbidden);
    }
    Ok(room)
}
