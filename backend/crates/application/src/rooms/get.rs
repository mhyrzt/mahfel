use crate::error::{ApplicationError, ErrorKind};
use crate::ports::{RoomStore, StoreError};
use mahfel_domain::{ActorId, Room, RoomId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetRoomError {
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

impl ApplicationError for GetRoomError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Store(_) => ErrorKind::Internal,
        }
    }
}

/// Return the room only when `actor` is a member.
///
/// A missing room and a non-member caller both return `None` so the transport
/// cannot leak room existence.
pub async fn get_room<R>(
    rooms: &R,
    id: RoomId,
    actor: ActorId,
) -> Result<Option<Room>, GetRoomError>
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
