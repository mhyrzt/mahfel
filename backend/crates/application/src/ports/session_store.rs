use super::{BoxFuture, StoreError};
use mahfel_domain::{RoomId, Session, SessionId};

/// Persistence for the session aggregate.
///
/// Methods return [`BoxFuture`] so the port is dyn-compatible.
pub trait SessionStore: Send + Sync {
    fn get(&self, id: SessionId) -> BoxFuture<'_, Result<Option<Session>, StoreError>>;

    fn insert<'a>(&'a self, session: &'a Session) -> BoxFuture<'a, Result<(), StoreError>>;

    fn save<'a>(&'a self, session: &'a Session) -> BoxFuture<'a, Result<(), StoreError>>;

    /// Sessions hosted by a room, newest first.
    fn list_for_room(&self, room_id: RoomId) -> BoxFuture<'_, Result<Vec<Session>, StoreError>>;
}
