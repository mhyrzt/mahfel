use crate::ports::{RoomStore, StoreError};
use mahfel_domain::{Room, RoomId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetRoomError {
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

pub async fn get_room<R>(rooms: &R, id: RoomId) -> Result<Option<Room>, GetRoomError>
where
    R: RoomStore,
{
    Ok(rooms.get(id).await?)
}
