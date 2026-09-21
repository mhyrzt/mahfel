//! Resolving an actor's membership in a room.
//!
//! Capability resolution needs the caller's room role, and every room-scoped
//! use case needs the same fact. Keeping it behind one port means a transport
//! cannot forget to load the membership before deciding, and a membership
//! change takes effect on the next check without a stale cache.

use super::{BoxFuture, RoomStore, StoreError};
use mahfel_domain::{ActorId, RoomId, RoomRole};

/// A resolved membership, or `None` when the actor has no membership.
pub type Membership = Option<RoomRole>;

/// Reads the caller's role in a room.
///
/// Deliberately read-only: membership changes go through [`RoomStore`] use
/// cases so their invariants (for example the last-manager rule) are enforced.
pub trait MembershipStore: Send + Sync {
    fn role_of(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<Membership, StoreError>>;
}

impl<T: RoomStore + ?Sized> MembershipStore for T {
    fn role_of(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<Membership, StoreError>> {
        Box::pin(async move {
            Ok(self
                .get_member(room_id, actor_id)
                .await?
                .map(|member| member.role))
        })
    }
}
