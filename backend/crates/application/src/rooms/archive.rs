use super::ManagerGuardError;
use crate::error::{ApplicationError, ErrorKind};
use crate::ports::{RoomStore, StoreError};
use mahfel_domain::{ActorId, Room, RoomError, RoomId};
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Debug, Error)]
pub enum ArchiveRoomError {
    #[error("room not found")]
    NotFound,
    #[error("permission denied")]
    Forbidden,
    #[error("invalid room state: {0}")]
    Domain(#[from] RoomError),
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

impl From<ManagerGuardError> for ArchiveRoomError {
    fn from(error: ManagerGuardError) -> Self {
        match error {
            ManagerGuardError::NotFound => Self::NotFound,
            ManagerGuardError::Forbidden => Self::Forbidden,
            ManagerGuardError::Store(error) => Self::Store(error),
        }
    }
}

impl ApplicationError for ArchiveRoomError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::NotFound => ErrorKind::NotFound,
            Self::Forbidden => ErrorKind::Forbidden,
            Self::Domain(_) => ErrorKind::Conflict,
            Self::Store(_) => ErrorKind::Internal,
        }
    }
}

/// Manager-only: archive a room. Archiving is terminal.
pub async fn archive_room<R>(
    rooms: &R,
    room_id: RoomId,
    actor: ActorId,
) -> Result<Room, ArchiveRoomError>
where
    R: RoomStore + ?Sized,
{
    let mut room = super::require_manager(rooms, room_id, actor).await?;
    room.archive(OffsetDateTime::now_utc())?;
    rooms.save(&room).await?;
    Ok(room)
}
