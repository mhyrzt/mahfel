use crate::RedisCoordination;
use mahfel_application::ports::{
    BoxFuture, ChatActivitySnapshot, ChatActivityStore, CoordinationError,
};
use mahfel_domain::{ActorId, RoomId};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::timeout;

const SNAPSHOT_SCRIPT: &str = "\
redis.call('ZREMRANGEBYSCORE', KEYS[1], '-inf', ARGV[1]); \
redis.call('ZREMRANGEBYSCORE', KEYS[2], '-inf', ARGV[1]); \
return {redis.call('ZCARD', KEYS[1]), redis.call('ZCARD', KEYS[2])}";

fn now_ms() -> Result<u64, CoordinationError> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| CoordinationError::Unexpected(error.to_string()))?;
    u64::try_from(elapsed.as_millis())
        .map_err(|error| CoordinationError::Unexpected(error.to_string()))
}

impl ChatActivityStore for RedisCoordination {
    fn heartbeat(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
        ttl: Duration,
    ) -> BoxFuture<'_, Result<(), CoordinationError>> {
        Box::pin(async move {
            let key = self.keys.key("chat-presence", &room_id.to_string());
            let ttl_ms = u64::try_from(ttl.as_millis()).unwrap_or(u64::MAX);
            let expiry = now_ms()?.saturating_add(ttl_ms);
            let mut connection = self.manager.clone();
            let result = timeout(
                self.command_timeout,
                redis::pipe()
                    .atomic()
                    .cmd("ZADD")
                    .arg(&key)
                    .arg(expiry)
                    .arg(actor_id.to_string())
                    .ignore()
                    .cmd("PEXPIRE")
                    .arg(&key)
                    .arg(ttl_ms.saturating_mul(2))
                    .ignore()
                    .query_async::<()>(&mut connection),
            )
            .await;
            self.finish(result).map_err(CoordinationError::from)
        })
    }

    fn set_typing(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
        active: bool,
        ttl: Duration,
    ) -> BoxFuture<'_, Result<(), CoordinationError>> {
        Box::pin(async move {
            let key = self.keys.key("chat-typing", &room_id.to_string());
            let ttl_ms = u64::try_from(ttl.as_millis()).unwrap_or(u64::MAX);
            let mut connection = self.manager.clone();
            let result = if active {
                let expiry = now_ms()?.saturating_add(ttl_ms);
                timeout(
                    self.command_timeout,
                    redis::pipe()
                        .atomic()
                        .cmd("ZADD")
                        .arg(&key)
                        .arg(expiry)
                        .arg(actor_id.to_string())
                        .ignore()
                        .cmd("PEXPIRE")
                        .arg(&key)
                        .arg(ttl_ms.saturating_mul(2))
                        .ignore()
                        .query_async::<()>(&mut connection),
                )
                .await
            } else {
                timeout(
                    self.command_timeout,
                    redis::cmd("ZREM")
                        .arg(&key)
                        .arg(actor_id.to_string())
                        .query_async::<()>(&mut connection),
                )
                .await
            };
            self.finish(result).map_err(CoordinationError::from)
        })
    }

    fn snapshot(
        &self,
        room_id: RoomId,
    ) -> BoxFuture<'_, Result<ChatActivitySnapshot, CoordinationError>> {
        Box::pin(async move {
            let presence = self.keys.key("chat-presence", &room_id.to_string());
            let typing = self.keys.key("chat-typing", &room_id.to_string());
            let now = now_ms()?;
            let mut connection = self.manager.clone();
            let result = timeout(
                self.command_timeout,
                redis::cmd("EVAL")
                    .arg(SNAPSHOT_SCRIPT)
                    .arg(2)
                    .arg(&presence)
                    .arg(&typing)
                    .arg(now)
                    .query_async::<(u64, u64)>(&mut connection),
            )
            .await;
            let (online_count, typing_count) =
                self.finish(result).map_err(CoordinationError::from)?;
            Ok(ChatActivitySnapshot {
                online_count,
                typing_count,
            })
        })
    }
}
