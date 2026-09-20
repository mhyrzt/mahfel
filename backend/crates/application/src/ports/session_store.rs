use super::StoreError;
use mahfel_domain::{RoomId, Session, SessionId};

/// Persistence for the session aggregate.
pub trait SessionStore: Send + Sync {
    fn get(
        &self,
        id: SessionId,
    ) -> impl std::future::Future<Output = Result<Option<Session>, StoreError>> + Send;
    fn insert(
        &self,
        session: &Session,
    ) -> impl std::future::Future<Output = Result<(), StoreError>> + Send;
    fn save(
        &self,
        session: &Session,
    ) -> impl std::future::Future<Output = Result<(), StoreError>> + Send;
    /// Sessions hosted by a room, newest first.
    fn list_for_room(
        &self,
        room_id: RoomId,
    ) -> impl std::future::Future<Output = Result<Vec<Session>, StoreError>> + Send;
}
