use std::time::Duration;

use mahfel_domain::NodeId;
use serde::{Deserialize, Serialize};

use crate::ports::{BoxFuture, CoordinationError};

/// Replaceable observation of one live media process. It is never durable truth.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MediaNodeRecord {
    pub node_id: NodeId,
    pub grpc_url: String,
    pub max_rooms: u32,
    pub active_workers: u32,
    pub active_rooms: u64,
    pub transports: u64,
    pub draining: bool,
}

/// Heartbeat writer and live-node reader used by the placement policy.
pub trait MediaNodeRegistry: Send + Sync {
    fn heartbeat<'a>(
        &'a self,
        record: &'a MediaNodeRecord,
        ttl: Duration,
    ) -> BoxFuture<'a, Result<(), CoordinationError>>;

    /// An outage is an error, never an empty healthy registry.
    fn live_nodes(&self) -> BoxFuture<'_, Result<Vec<MediaNodeRecord>, CoordinationError>>;

    /// Read one assigned node with a fresh heartbeat. Draining nodes remain
    /// readable for their existing sessions.
    fn live_node<'a>(
        &'a self,
        node_id: &'a NodeId,
    ) -> BoxFuture<'a, Result<Option<MediaNodeRecord>, CoordinationError>>;
}
