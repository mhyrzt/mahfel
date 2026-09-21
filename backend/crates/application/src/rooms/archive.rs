use super::RoomOperationError;
use crate::ports::RoomStore;
use mahfel_domain::{ActorId, Room, RoomId};
use time::OffsetDateTime;

/// Manager-only: archive a room. Archiving is terminal.
pub async fn archive_room<R>(
    rooms: &R,
    room_id: RoomId,
    actor: ActorId,
) -> Result<Room, RoomOperationError>
where
    R: RoomStore + ?Sized,
{
    let mut room = super::require_manager(rooms, room_id, actor).await?;
    room.archive(OffsetDateTime::now_utc())?;
    rooms.save(&room).await?;
    Ok(room)
}
