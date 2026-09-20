use super::ManagerGuardError;
use crate::error::{ApplicationError, ErrorKind};
use crate::ports::{RoomStore, StoreError};
use mahfel_domain::{ActorId, Room, RoomError, RoomId, RoomTitle, RoomVisibility, ValidationError};
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Debug, Clone, Default)]
pub struct UpdateRoom {
    pub title: Option<String>,
    pub visibility: Option<RoomVisibility>,
}

#[derive(Debug, Error)]
pub enum UpdateRoomError {
    #[error("room not found")]
    NotFound,
    #[error("permission denied")]
    Forbidden,
    #[error("invalid room title: {0}")]
    InvalidTitle(#[from] ValidationError),
    #[error("invalid room state: {0}")]
    Domain(#[from] RoomError),
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

impl From<ManagerGuardError> for UpdateRoomError {
    fn from(error: ManagerGuardError) -> Self {
        match error {
            ManagerGuardError::NotFound => Self::NotFound,
            ManagerGuardError::Forbidden => Self::Forbidden,
            ManagerGuardError::Store(error) => Self::Store(error),
        }
    }
}

impl ApplicationError for UpdateRoomError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::NotFound => ErrorKind::NotFound,
            Self::Forbidden => ErrorKind::Forbidden,
            Self::InvalidTitle(_) => ErrorKind::Unprocessable,
            Self::Domain(_) => ErrorKind::Conflict,
            Self::Store(_) => ErrorKind::Internal,
        }
    }
}

/// Manager-only: update room settings. Changing visibility preserves existing
/// members and their roles.
pub async fn update_settings<R>(
    rooms: &R,
    room_id: RoomId,
    actor: ActorId,
    command: UpdateRoom,
) -> Result<Room, UpdateRoomError>
where
    R: RoomStore,
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
