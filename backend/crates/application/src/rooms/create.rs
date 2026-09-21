use super::RoomOperationError;
use crate::ports::RoomStore;
use mahfel_domain::{ActorId, Room, RoomMember, RoomRole, RoomTitle, RoomVisibility};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct CreateRoom {
    pub title: String,
    pub visibility: RoomVisibility,
    pub created_by: ActorId,
}

/// Create a room and make its creator the first manager, atomically.
pub async fn create_room<R>(rooms: &R, command: CreateRoom) -> Result<Room, RoomOperationError>
where
    R: RoomStore + ?Sized,
{
    let title = RoomTitle::new(command.title)?;
    let now = OffsetDateTime::now_utc();
    let room = Room::create(title, command.created_by, command.visibility, now);
    let manager = RoomMember::new(room.id, command.created_by, RoomRole::Manager, now);
    rooms.create_with_manager(&room, &manager).await?;
    Ok(room)
}
