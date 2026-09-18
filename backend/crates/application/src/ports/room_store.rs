use super::StoreError;
use mahfel_domain::{Room, RoomId};

pub trait RoomStore: Send + Sync {
    fn get(
        &self,
        id: RoomId,
    ) -> impl std::future::Future<Output = Result<Option<Room>, StoreError>> + Send;
    fn insert(
        &self,
        room: &Room,
    ) -> impl std::future::Future<Output = Result<(), StoreError>> + Send;
    fn save(&self, room: &Room)
    -> impl std::future::Future<Output = Result<(), StoreError>> + Send;
}
