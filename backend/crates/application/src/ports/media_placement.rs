use std::time::Duration;

use mahfel_domain::{NodeId, RoomId, SessionId};
use time::OffsetDateTime;

use crate::ports::{BoxFuture, CoordinationError, StoreError};

/// The durable, immutable decision for one session.
#[derive(Clone, Debug)]
pub struct MediaPlacement {
    pub session_id: SessionId,
    pub room_id: RoomId,
    pub node_id: NodeId,
    pub max_rooms: u32,
    pub active_rooms: u64,
    pub reason: String,
    pub decided_at: OffsetDateTime,
}

/// PostgreSQL is the source of truth after first placement. A unique session
/// key resolves a race even if a Redis lease expires before a database write.
pub trait MediaPlacementStore: Send + Sync {
    fn get(
        &self,
        session_id: SessionId,
    ) -> BoxFuture<'_, Result<Option<MediaPlacement>, StoreError>>;

    /// Return the winning row, whether this call inserted it or another won.
    fn insert_or_get<'a>(
        &'a self,
        placement: &'a MediaPlacement,
    ) -> BoxFuture<'a, Result<MediaPlacement, StoreError>>;
}

/// Atomic first-placement and room-slot reservation across API replicas.
pub trait MediaPlacementLease: Send + Sync {
    fn acquire<'a>(
        &'a self,
        session_id: SessionId,
        candidates: &'a [NodeId],
        ttl: Duration,
    ) -> BoxFuture<'a, Result<Option<NodeId>, CoordinationError>>;
}
