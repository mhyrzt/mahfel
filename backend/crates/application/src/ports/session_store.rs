use super::{BoxFuture, StoreError};
use mahfel_domain::{RoomId, Session, SessionId};
use uuid::Uuid;

/// A database-fenced start decision. An unrecorded session is committed in the
/// same transaction; a recorded one must pass capture preflight first.
pub enum SessionStartClaim {
    Started(Session),
    RecordingLease(Uuid),
}

/// Which advisory fields a PATCH request supplied. A supplied null clears its
/// field; an absent field must not overwrite a concurrent edit to that field.
#[derive(Clone, Copy, Debug, Default)]
pub struct SessionDetailsFields {
    pub auto_record: bool,
    pub title: bool,
    pub scheduled_start_at: bool,
    pub scheduled_duration_minutes: bool,
}

/// Persistence for the session aggregate.
///
/// Methods return [`BoxFuture`] so the port is dyn-compatible.
pub trait SessionStore: Send + Sync {
    fn claim_start(&self, id: SessionId) -> BoxFuture<'_, Result<SessionStartClaim, StoreError>> {
        Box::pin(async move {
            let _ = id;
            Err(StoreError::Unexpected(
                "recording start gate unavailable".into(),
            ))
        })
    }
    fn commit_recorded_start(
        &self,
        id: SessionId,
        token: Uuid,
    ) -> BoxFuture<'_, Result<Session, StoreError>> {
        Box::pin(async move {
            let _ = (id, token);
            Err(StoreError::Unexpected(
                "recording start gate unavailable".into(),
            ))
        })
    }
    fn release_recorded_start(
        &self,
        id: SessionId,
        token: Uuid,
    ) -> BoxFuture<'_, Result<(), StoreError>> {
        Box::pin(async move {
            let _ = (id, token);
            Err(StoreError::Unexpected(
                "recording start gate unavailable".into(),
            ))
        })
    }

    fn get(&self, id: SessionId) -> BoxFuture<'_, Result<Option<Session>, StoreError>>;

    fn insert<'a>(&'a self, session: &'a Session) -> BoxFuture<'a, Result<(), StoreError>>;

    fn save<'a>(&'a self, session: &'a Session) -> BoxFuture<'a, Result<(), StoreError>>;

    /// Persist display/schedule details without overwriting a concurrent
    /// lifecycle transition. Adapters should return the resulting row.
    fn save_details<'a>(
        &'a self,
        session: &'a Session,
        fields: SessionDetailsFields,
    ) -> BoxFuture<'a, Result<Session, StoreError>>;

    /// Sessions hosted by a room, newest first.
    /// Bounded session history, optionally filtered by lifecycle status.
    fn list_for_room_page(
        &self,
        room_id: RoomId,
        status: Option<mahfel_domain::SessionStatus>,
        limit: u64,
        offset: u64,
    ) -> BoxFuture<'_, Result<Vec<Session>, StoreError>>;

    fn list_for_room(&self, room_id: RoomId) -> BoxFuture<'_, Result<Vec<Session>, StoreError>>;
}
