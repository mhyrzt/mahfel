use crate::error::{ApplicationError, ErrorKind};
use crate::ports::{RoomStore, StoreError};
use mahfel_domain::{
    ActorId, Room, RoomMember, RoomRole, RoomTitle, RoomVisibility, ValidationError,
};
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct CreateRoom {
    pub title: String,
    pub visibility: RoomVisibility,
    pub created_by: ActorId,
}

#[derive(Debug, Error)]
pub enum CreateRoomError {
    #[error("invalid room title: {0}")]
    InvalidTitle(#[from] ValidationError),
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

impl ApplicationError for CreateRoomError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::InvalidTitle(_) => ErrorKind::Unprocessable,
            Self::Store(_) => ErrorKind::Internal,
        }
    }
}

/// Create a room and make its creator the first manager, atomically.
pub async fn create_room<R>(rooms: &R, command: CreateRoom) -> Result<Room, CreateRoomError>
where
    R: RoomStore,
{
    let title = RoomTitle::new(command.title)?;
    let now = OffsetDateTime::now_utc();
    let room = Room::create(title, command.created_by, command.visibility, now);
    let manager = RoomMember::new(room.id, command.created_by, RoomRole::Manager, now);
    rooms.create_with_manager(&room, &manager).await?;
    Ok(room)
}
