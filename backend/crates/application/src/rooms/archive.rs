use crate::ports::{RoomStore, StoreError};
use mahfel_domain::{Room, RoomError, RoomId};
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct ArchiveRoom {
    pub room_id: RoomId,
}

#[derive(Debug, Error)]
pub enum ArchiveRoomError {
    #[error("room not found")]
    NotFound,
    #[error("invalid room state: {0}")]
    Domain(#[from] RoomError),
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

pub async fn execute<R>(rooms: &R, command: ArchiveRoom) -> Result<Room, ArchiveRoomError>
where
    R: RoomStore,
{
    let mut room = rooms
        .get(command.room_id)
        .await?
        .ok_or(ArchiveRoomError::NotFound)?;
    room.archive(OffsetDateTime::now_utc())?;
    rooms.save(&room).await?;
    Ok(room)
}
