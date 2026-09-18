use crate::ports::{RoomStore, StoreError};
use mahfel_domain::{Room, RoomTitle, UserId, ValidationError};
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct CreateRoom {
    pub title: String,
    pub created_by: UserId,
}

#[derive(Debug, Error)]
pub enum CreateRoomError {
    #[error("invalid room title: {0}")]
    InvalidTitle(#[from] ValidationError),
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

pub async fn execute<R>(rooms: &R, command: CreateRoom) -> Result<Room, CreateRoomError>
where
    R: RoomStore,
{
    let title = RoomTitle::new(command.title)?;
    let room = Room::create(title, command.created_by, OffsetDateTime::now_utc());
    rooms.insert(&room).await?;
    Ok(room)
}
