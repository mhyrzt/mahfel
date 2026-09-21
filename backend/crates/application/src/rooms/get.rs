use super::RoomOperationError;
use crate::ports::RoomStore;
use mahfel_domain::{ActorId, Room, RoomId};

/// Return the room only when `actor` is a member.
///
/// A missing room and a non-member caller both return `None` so the transport
/// cannot leak room existence.
pub async fn get_room<R>(
    rooms: &R,
    id: RoomId,
    actor: ActorId,
) -> Result<Option<Room>, RoomOperationError>
where
    R: RoomStore + ?Sized,
{
    let Some(room) = rooms.get(id).await? else {
        return Ok(None);
    };
    if rooms.get_member(id, actor).await?.is_none() {
        return Ok(None);
    }
    Ok(Some(room))
}
