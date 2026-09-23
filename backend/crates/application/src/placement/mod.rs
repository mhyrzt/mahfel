mod policy;
#[cfg(test)]
mod tests;

use std::time::Duration;

use mahfel_domain::{RoomId, SessionId};
use thiserror::Error;
use time::OffsetDateTime;

use crate::ports::{
    CoordinationError, MediaNodeRecord, MediaNodeRegistry, MediaPlacement, MediaPlacementLease,
    MediaPlacementStore, StoreError,
};

const RESERVATION_TTL: Duration = Duration::from_secs(20);

#[derive(Debug, Error)]
pub enum PlacementError {
    #[error("media placement unavailable")]
    Unavailable,
    #[error("placement storage failure: {0}")]
    Store(#[from] StoreError),
    #[error("placement coordination failure: {0}")]
    Coordination(#[from] CoordinationError),
}

/// The node and its private endpoint for a session. The endpoint is read from
/// a fresh registry heartbeat, never persisted as an authority for liveness.
#[derive(Clone, Debug)]
pub struct PlacedNode {
    pub placement: MediaPlacement,
    pub node: MediaNodeRecord,
}

/// Resolve the immutable decision, or atomically make the first one.
pub async fn place_session<S, L, R>(
    store: &S,
    lease: &L,
    registry: &R,
    session_id: SessionId,
    room_id: RoomId,
) -> Result<PlacedNode, PlacementError>
where
    S: MediaPlacementStore + ?Sized,
    L: MediaPlacementLease + ?Sized,
    R: MediaNodeRegistry + ?Sized,
{
    if let Some(placement) = store.get(session_id).await? {
        return live_placement(registry, placement, room_id).await;
    }

    let nodes = registry.live_nodes().await?;
    let candidates = policy::ordered_candidates(session_id, &nodes);
    if candidates.is_empty() {
        return Err(PlacementError::Unavailable);
    }
    let winner = lease
        .acquire(session_id, &candidates, RESERVATION_TTL)
        .await?
        .ok_or(PlacementError::Unavailable)?;
    let node = nodes
        .iter()
        .find(|node| node.node_id == winner)
        .ok_or(PlacementError::Unavailable)?;
    let decision = MediaPlacement {
        session_id,
        room_id,
        node_id: winner,
        max_rooms: node.max_rooms,
        active_rooms: node.active_rooms,
        reason: "weighted_available_capacity".to_owned(),
        decided_at: OffsetDateTime::now_utc(),
    };
    let placement = store.insert_or_get(&decision).await?;
    live_placement(registry, placement, room_id).await
}

/// A read-only sticky lookup for media operations after a join.
pub async fn lookup_session<S, R>(
    store: &S,
    registry: &R,
    session_id: SessionId,
    room_id: RoomId,
) -> Result<PlacedNode, PlacementError>
where
    S: MediaPlacementStore + ?Sized,
    R: MediaNodeRegistry + ?Sized,
{
    let placement = store
        .get(session_id)
        .await?
        .ok_or(PlacementError::Unavailable)?;
    live_placement(registry, placement, room_id).await
}

async fn live_placement<R>(
    registry: &R,
    placement: MediaPlacement,
    room_id: RoomId,
) -> Result<PlacedNode, PlacementError>
where
    R: MediaNodeRegistry + ?Sized,
{
    if placement.room_id != room_id {
        return Err(PlacementError::Unavailable);
    }
    let node = registry
        .live_node(&placement.node_id)
        .await?
        .ok_or(PlacementError::Unavailable)?;
    Ok(PlacedNode { placement, node })
}
