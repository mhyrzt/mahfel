use super::{BoxFuture, StoreError};
use mahfel_domain::{
    ActorId, AdmissionRequest, AdmissionRequestId, AdmissionRequestStatus, SessionId,
};

/// Persistence for pending admission requests (M4-06).
///
/// A pending request is durable rather than ephemeral: the waiting principal is
/// looking at a waiting screen, so losing the row to a cache flush would
/// silently drop them. Whether they are still connected is the ephemeral part
/// and lives in the realtime registry. Methods return [`BoxFuture`] so the port
/// is dyn-compatible (ADR 0023).
pub trait AdmissionStore: Send + Sync {
    /// Insert a request. Returns [`StoreError::Conflict`] when the actor
    /// already has a pending request for the session.
    fn insert<'a>(&'a self, request: &'a AdmissionRequest)
    -> BoxFuture<'a, Result<(), StoreError>>;

    /// The actor's outstanding request for one session, if any.
    fn get_pending(
        &self,
        session_id: SessionId,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<Option<AdmissionRequest>, StoreError>>;

    fn get(
        &self,
        id: AdmissionRequestId,
    ) -> BoxFuture<'_, Result<Option<AdmissionRequest>, StoreError>>;

    /// One session's queue, oldest first.
    fn list_pending(
        &self,
        session_id: SessionId,
        limit: u32,
    ) -> BoxFuture<'_, Result<Vec<AdmissionRequest>, StoreError>>;

    /// How many principals are waiting in one session, for the cap.
    fn count_pending(&self, session_id: SessionId) -> BoxFuture<'_, Result<u64, StoreError>>;

    /// Apply a decision only if the request is still pending.
    ///
    /// Returns whether a row changed, so two concurrent decisions cannot both
    /// report success, and a repeat never overwrites the first decision.
    fn decide_if_pending(
        &self,
        id: AdmissionRequestId,
        status: AdmissionRequestStatus,
        decided_by: Option<ActorId>,
        decided_at: time::OffsetDateTime,
    ) -> BoxFuture<'_, Result<bool, StoreError>>;

    /// Retire every pending request in a session because the session ended.
    ///
    /// Returns how many were retired, so the caller can audit a real change
    /// rather than a repeat. Idempotent.
    fn expire_pending_for_session(
        &self,
        session_id: SessionId,
        expired_at: time::OffsetDateTime,
    ) -> BoxFuture<'_, Result<u64, StoreError>>;
}
