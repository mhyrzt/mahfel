//! Session authorization guards shared by the session use cases.

use crate::error::GuardError;
use crate::ports::RoomStore;
use mahfel_domain::{ActorId, Room, RoomId, RoomStatus, RoomVisibility};

/// Load the room and require the actor to be allowed to join it.
///
/// Deny-by-default: a manager or member may always join, and a non-member may
/// join only a `Public` room. Admission policy (lobby) is layered on top of
/// this by M4-06 through the same path.
pub(crate) async fn require_joinable<R>(
    rooms: &R,
    room_id: RoomId,
    actor: ActorId,
) -> Result<Room, GuardError>
where
    R: RoomStore + ?Sized,
{
    let room = rooms.get(room_id).await?.ok_or(GuardError::NotFound)?;
    if room.status != RoomStatus::Active {
        return Err(GuardError::Forbidden);
    }
    if rooms.get_member(room_id, actor).await?.is_some() {
        return Ok(room);
    }
    if room.visibility == RoomVisibility::Public {
        return Ok(room);
    }
    Err(GuardError::Forbidden)
}
