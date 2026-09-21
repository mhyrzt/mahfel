use super::{BoxFuture, StoreError};
use mahfel_domain::{ActorId, Participant, ParticipantId, SessionId};

/// Persistence for durable session participation.
///
/// Identity is keyed by `(session_id, actor_id)`. Methods return [`BoxFuture`]
/// so the port is dyn-compatible (ADR 0023).
pub trait ParticipantStore: Send + Sync {
    fn get(&self, id: ParticipantId) -> BoxFuture<'_, Result<Option<Participant>, StoreError>>;

    /// The actor's active participant, the reconnect check.
    fn get_active(
        &self,
        session_id: SessionId,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<Option<Participant>, StoreError>>;

    /// Returns [`StoreError::Conflict`] when the actor already has an active
    /// participant for the session.
    fn insert<'a>(&'a self, participant: &'a Participant) -> BoxFuture<'a, Result<(), StoreError>>;

    fn save<'a>(&'a self, participant: &'a Participant) -> BoxFuture<'a, Result<(), StoreError>>;

    /// Participants of a session, oldest first.
    fn list_for_session(
        &self,
        session_id: SessionId,
    ) -> BoxFuture<'_, Result<Vec<Participant>, StoreError>>;
}
