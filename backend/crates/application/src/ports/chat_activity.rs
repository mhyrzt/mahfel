use super::{BoxFuture, CoordinationError};
use mahfel_domain::{ActorId, RoomId};
use std::time::Duration;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ChatActivitySnapshot {
    pub online_count: u64,
    pub typing_count: u64,
}

/// Replaceable room activity. Every record expires without a durable write.
pub trait ChatActivityStore: Send + Sync {
    fn heartbeat(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
        ttl: Duration,
    ) -> BoxFuture<'_, Result<(), CoordinationError>>;

    fn set_typing(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
        active: bool,
        ttl: Duration,
    ) -> BoxFuture<'_, Result<(), CoordinationError>>;

    fn snapshot(
        &self,
        room_id: RoomId,
    ) -> BoxFuture<'_, Result<ChatActivitySnapshot, CoordinationError>>;
}
