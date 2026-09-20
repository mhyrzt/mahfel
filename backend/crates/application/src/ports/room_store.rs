use super::StoreError;
use mahfel_domain::{ActorId, Room, RoomId, RoomMember};

/// Persistence for the room aggregate: the room itself and its memberships.
///
/// Rooms and memberships share one port because creating a room must persist
/// the creator's manager membership atomically, and membership changes are
/// room-scoped invariants.
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

    /// Persist a room and its creator's manager membership in one transaction.
    fn create_with_manager(
        &self,
        room: &Room,
        manager: &RoomMember,
    ) -> impl std::future::Future<Output = Result<(), StoreError>> + Send;

    /// Rooms the user is a member of, newest first.
    fn list_for_member(
        &self,
        user_id: ActorId,
        limit: u64,
        offset: u64,
    ) -> impl std::future::Future<Output = Result<Vec<Room>, StoreError>> + Send;

    fn get_member(
        &self,
        room_id: RoomId,
        user_id: ActorId,
    ) -> impl std::future::Future<Output = Result<Option<RoomMember>, StoreError>> + Send;

    fn list_members(
        &self,
        room_id: RoomId,
    ) -> impl std::future::Future<Output = Result<Vec<RoomMember>, StoreError>> + Send;

    /// Insert a membership. Returns [`StoreError::Conflict`] when the user is
    /// already a member.
    fn add_member(
        &self,
        member: &RoomMember,
    ) -> impl std::future::Future<Output = Result<(), StoreError>> + Send;

    /// Remove a membership, refusing to remove the last manager.
    ///
    /// Returns `true` when a row was removed, `false` when the membership did
    /// not exist or the target is the last manager.
    fn remove_member(
        &self,
        room_id: RoomId,
        user_id: ActorId,
    ) -> impl std::future::Future<Output = Result<bool, StoreError>> + Send;
}
