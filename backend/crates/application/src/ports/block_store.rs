use super::{BoxFuture, StoreError};
use mahfel_domain::{ActorId, Block, BlockId, BlockScope, RoomId, SessionId};

/// Persistence for durable blocks.
///
/// A block is durable rather than ephemeral: its whole purpose is to outlive a
/// participation, so a cache flush or a node restart must not let a blocked
/// principal back in. Methods return [`BoxFuture`] so the port is
/// dyn-compatible (ADR 0023).
pub trait BlockStore: Send + Sync {
    /// The active block barring `actor_id` from this scope, if any.
    ///
    /// A block that has expired is not active and is not returned; it was never
    /// lifted, it simply ran out.
    fn get_active(
        &self,
        scope: BlockScope,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<Option<Block>, StoreError>>;

    /// Insert a block. Returns [`StoreError::Conflict`] when an active block
    /// already exists for the same target, because re-blocking extends rather
    /// than stacking.
    fn insert<'a>(&'a self, block: &'a Block) -> BoxFuture<'a, Result<(), StoreError>>;

    /// Extend an active block to a new expiry, where `None` makes it
    /// indefinite. Returns whether a row changed.
    fn extend(
        &self,
        id: BlockId,
        expires_at: Option<time::OffsetDateTime>,
    ) -> BoxFuture<'_, Result<bool, StoreError>>;

    /// Lift a block. Returns whether a row changed, so a repeat lift cannot
    /// overwrite who lifted it.
    fn lift(
        &self,
        id: BlockId,
        lifted_by: ActorId,
        lifted_at: time::OffsetDateTime,
    ) -> BoxFuture<'_, Result<bool, StoreError>>;

    /// Active blocks imposed on a room, for a manager's view.
    fn list_active_for_room(
        &self,
        room_id: RoomId,
        limit: u32,
    ) -> BoxFuture<'_, Result<Vec<Block>, StoreError>>;

    /// Active blocks imposed on one session.
    fn list_active_for_session(
        &self,
        session_id: SessionId,
        limit: u32,
    ) -> BoxFuture<'_, Result<Vec<Block>, StoreError>>;
}
