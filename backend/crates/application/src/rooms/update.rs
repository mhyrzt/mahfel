use super::RoomOperationError;
use crate::ports::RoomStore;
use mahfel_domain::{ActorId, Room, RoomId, RoomTitle, RoomVisibility};
use time::OffsetDateTime;

#[derive(Debug, Clone, Default)]
pub struct UpdateRoom {
    pub title: Option<String>,
    pub visibility: Option<RoomVisibility>,
}

/// Manager-only: update room settings. Changing visibility preserves existing
/// members and their roles.
pub async fn update_settings<R>(
    rooms: &R,
    room_id: RoomId,
    actor: ActorId,
    command: UpdateRoom,
) -> Result<Room, RoomOperationError>
where
    R: RoomStore + ?Sized,
{
    let mut room = super::require_manager(rooms, room_id, actor).await?;
    let now = OffsetDateTime::now_utc();
    if let Some(title) = command.title {
        room.rename(RoomTitle::new(title)?, now)?;
    }
    if let Some(visibility) = command.visibility {
        room.set_visibility(visibility, now)?;
    }
    rooms.save(&room).await?;
    Ok(room)
}
