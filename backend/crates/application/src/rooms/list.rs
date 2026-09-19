use crate::error::{ApplicationError, ErrorKind};
use crate::ports::{RoomStore, StoreError};
use mahfel_domain::{Room, UserId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ListRoomsError {
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

impl ApplicationError for ListRoomsError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Store(_) => ErrorKind::Internal,
        }
    }
}

/// List the rooms the actor has joined. There is no discovery: a room appears
/// only after a membership exists.
pub async fn list_joined<R>(
    rooms: &R,
    actor: UserId,
    limit: u64,
    offset: u64,
) -> Result<Vec<Room>, ListRoomsError>
where
    R: RoomStore,
{
    Ok(rooms.list_for_member(actor, limit, offset).await?)
}
