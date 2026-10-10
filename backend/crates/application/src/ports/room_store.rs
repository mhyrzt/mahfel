use super::{BoxFuture, StoreError};
use mahfel_domain::{ActorId, Room, RoomId, RoomMember};

/// Persistence for the room aggregate: the room itself and its memberships.
///
/// Rooms and memberships share one port because creating a room must persist
/// the creator's manager membership atomically, and membership changes are
/// room-scoped invariants.
///
/// Methods return [`BoxFuture`] so the port is dyn-compatible and the
/// composition root can hold one `Arc<dyn RoomStore>`.
pub trait RoomStore: Send + Sync {
    fn get(&self, id: RoomId) -> BoxFuture<'_, Result<Option<Room>, StoreError>>;

    fn insert<'a>(&'a self, room: &'a Room) -> BoxFuture<'a, Result<(), StoreError>>;

    fn save<'a>(&'a self, room: &'a Room) -> BoxFuture<'a, Result<(), StoreError>>;

    /// Persist a room and its creator's manager membership in one transaction.
    fn create_with_manager<'a>(
        &'a self,
        room: &'a Room,
        manager: &'a RoomMember,
    ) -> BoxFuture<'a, Result<(), StoreError>>;

    /// Rooms the actor is a member of, newest first.
    fn list_for_member(
        &self,
        actor_id: ActorId,
        limit: u64,
        offset: u64,
    ) -> BoxFuture<'_, Result<Vec<Room>, StoreError>>;

    fn get_member(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<Option<RoomMember>, StoreError>>;

    fn list_members(&self, room_id: RoomId) -> BoxFuture<'_, Result<Vec<RoomMember>, StoreError>>;

    /// Insert a membership. Returns [`StoreError::Conflict`] when the actor is
    /// already a member.
    fn add_member<'a>(&'a self, member: &'a RoomMember) -> BoxFuture<'a, Result<(), StoreError>>;

    /// Update an existing membership, preserving the last manager atomically.
    /// Returns false if the membership disappeared or the last manager would be demoted.
    fn update_member<'a>(
        &'a self,
        member: &'a RoomMember,
    ) -> BoxFuture<'a, Result<bool, StoreError>>;

    /// Remove a membership, refusing to remove the last manager.
    ///
    /// Returns `true` when a row was removed, `false` when the membership did
    /// not exist or the target is the last manager.
    fn remove_member(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<bool, StoreError>>;
}
