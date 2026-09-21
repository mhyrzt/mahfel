use super::RoomOperationError;
use crate::ports::RoomStore;
use mahfel_domain::{ActorId, Room};

/// List the rooms the actor has joined. There is no discovery: a room appears
/// only after a membership exists.
pub async fn list_joined<R>(
    rooms: &R,
    actor: ActorId,
    limit: u64,
    offset: u64,
) -> Result<Vec<Room>, RoomOperationError>
where
    R: RoomStore + ?Sized,
{
    Ok(rooms.list_for_member(actor, limit, offset).await?)
}
