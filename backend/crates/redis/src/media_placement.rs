use std::time::Duration;

use mahfel_application::ports::{BoxFuture, CoordinationError, MediaPlacementLease};
use mahfel_domain::{NodeId, SessionId};

use crate::RedisCoordination;

// Candidate order is chosen by the application policy. This script rechecks
// liveness, drain state, and capacity at the same instant it claims a session.
const ACQUIRE_SCRIPT: &str = r#"
local existing = redis.call('GET', KEYS[1])
if existing then return existing end
local now = redis.call('TIME')
local now_ms = now[1] * 1000 + math.floor(now[2] / 1000)
local ttl = tonumber(ARGV[1])
for i = 3, #ARGV do
    local id = ARGV[i]
    local expires = redis.call('ZSCORE', KEYS[2], id)
    if expires and tonumber(expires) > now_ms then
        local record = KEYS[3] .. id
        if redis.call('HGET', record, 'draining') ~= '1' then
            local payload = redis.call('HGET', record, 'payload')
            if payload then
                local node = cjson.decode(payload)
                local capacity = tonumber(node.max_rooms) or 0
                local active = tonumber(node.active_rooms) or 0
                local reservations = KEYS[4] .. id
                redis.call('ZREMRANGEBYSCORE', reservations, '-inf', now_ms)
                if tonumber(node.active_workers) > 0 and capacity > 0 and active + redis.call('ZCARD', reservations) < capacity then
                    redis.call('SET', KEYS[1], id, 'PX', ttl)
                    redis.call('ZADD', reservations, now_ms + ttl, ARGV[2])
                    redis.call('PEXPIRE', reservations, ttl)
                    return id
                end
            end
        end
    end
end
return ''
"#;

impl MediaPlacementLease for RedisCoordination {
    fn acquire<'a>(
        &'a self,
        session_id: SessionId,
        candidates: &'a [NodeId],
        ttl: Duration,
    ) -> BoxFuture<'a, Result<Option<NodeId>, CoordinationError>> {
        Box::pin(async move {
            let ttl_ms = u64::try_from(ttl.as_millis())
                .map_err(|error| CoordinationError::Unexpected(error.to_string()))?;
            if ttl_ms == 0 {
                return Err(CoordinationError::Unexpected("zero placement TTL".into()));
            }
            let mut command = redis::cmd("EVAL");
            command.arg(ACQUIRE_SCRIPT).arg(4);
            command.arg(self.keys.key("media_placement", &session_id.to_string()));
            command.arg(self.keys.key("media_nodes", "live"));
            command.arg(format!("{}:", self.keys.key("media_nodes", "record")));
            command.arg(format!("{}:", self.keys.key("media_reservations", "node")));
            command.arg(ttl_ms).arg(session_id.to_string());
            for candidate in candidates {
                command.arg(candidate.as_str());
            }
            let mut connection = self.manager.clone();
            let result = tokio::time::timeout(
                self.command_timeout,
                command.query_async::<String>(&mut connection),
            )
            .await;
            let winner = self.finish(result).map_err(CoordinationError::from)?;
            if winner.is_empty() {
                Ok(None)
            } else {
                NodeId::from_deployment(winner)
                    .map(Some)
                    .ok_or_else(|| CoordinationError::Unexpected("invalid placed node id".into()))
            }
        })
    }
}
