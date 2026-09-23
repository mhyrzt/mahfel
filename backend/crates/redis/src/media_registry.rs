use std::time::Duration;

use mahfel_application::ports::{BoxFuture, CoordinationError, MediaNodeRecord, MediaNodeRegistry};

use crate::RedisCoordination;

const MAX_LIVE_NODES: isize = 1_000;
const HEARTBEAT_SCRIPT: &str = r#"
local old_draining = redis.call('HGET', KEYS[1], 'draining')
local draining = ARGV[2]
if old_draining == '1' then draining = '1' end
redis.call('HSET', KEYS[1], 'payload', ARGV[1], 'draining', draining)
redis.call('PEXPIRE', KEYS[1], ARGV[3])
local now = redis.call('TIME')
local now_ms = now[1] * 1000 + math.floor(now[2] / 1000)
redis.call('ZREMRANGEBYSCORE', KEYS[2], '-inf', now_ms)
redis.call('ZADD', KEYS[2], now_ms + tonumber(ARGV[3]), ARGV[4])
redis.call('PEXPIRE', KEYS[2], ARGV[3])
return 1
"#;
const LIVE_NODE_SCRIPT: &str = r#"
local now = redis.call('TIME')
local now_ms = now[1] * 1000 + math.floor(now[2] / 1000)
local expires = redis.call('ZSCORE', KEYS[2], ARGV[1])
if not expires or tonumber(expires) <= now_ms then return {'', ''} end
local values = redis.call('HMGET', KEYS[1], 'payload', 'draining')
return {values[1] or '', values[2] or ''}
"#;

impl RedisCoordination {
    fn media_node_key(&self, node_id: &str) -> String {
        self.keys.key("media_nodes:record", node_id)
    }

    fn media_nodes_index(&self) -> String {
        self.keys.key("media_nodes", "live")
    }
}

impl MediaNodeRegistry for RedisCoordination {
    fn live_node<'a>(
        &'a self,
        node_id: &'a mahfel_domain::NodeId,
    ) -> BoxFuture<'a, Result<Option<MediaNodeRecord>, CoordinationError>> {
        Box::pin(async move {
            let mut connection = self.manager.clone();
            let result = tokio::time::timeout(
                self.command_timeout,
                redis::cmd("EVAL")
                    .arg(LIVE_NODE_SCRIPT)
                    .arg(2)
                    .arg(self.media_node_key(node_id.as_str()))
                    .arg(self.media_nodes_index())
                    .arg(node_id.as_str())
                    .query_async::<(String, String)>(&mut connection),
            )
            .await;
            let (payload, draining) = self.finish(result).map_err(CoordinationError::from)?;
            if payload.is_empty() {
                return Ok(None);
            }
            let mut node: MediaNodeRecord = serde_json::from_str(&payload)
                .map_err(|error| CoordinationError::Unexpected(error.to_string()))?;
            if node.node_id != *node_id {
                return Err(CoordinationError::Unexpected(
                    "media node registry identity mismatch".into(),
                ));
            }
            node.draining = draining == "1";
            Ok(Some(node))
        })
    }

    fn heartbeat<'a>(
        &'a self,
        record: &'a MediaNodeRecord,
        ttl: Duration,
    ) -> BoxFuture<'a, Result<(), CoordinationError>> {
        Box::pin(async move {
            let payload = serde_json::to_vec(record)
                .map_err(|error| CoordinationError::Unexpected(error.to_string()))?;
            let ttl_ms = u64::try_from(ttl.as_millis())
                .map_err(|error| CoordinationError::Unexpected(error.to_string()))?;
            if ttl_ms == 0 {
                return Err(CoordinationError::Unexpected("zero heartbeat TTL".into()));
            }
            let mut connection = self.manager.clone();
            let result = tokio::time::timeout(
                self.command_timeout,
                redis::cmd("EVAL")
                    .arg(HEARTBEAT_SCRIPT)
                    .arg(2)
                    .arg(self.media_node_key(record.node_id.as_str()))
                    .arg(self.media_nodes_index())
                    .arg(payload)
                    .arg(if record.draining { "1" } else { "0" })
                    .arg(ttl_ms)
                    .arg(record.node_id.as_str())
                    .query_async::<i64>(&mut connection),
            )
            .await;
            self.finish(result)
                .map(|_| ())
                .map_err(CoordinationError::from)
        })
    }

    fn live_nodes(&self) -> BoxFuture<'_, Result<Vec<MediaNodeRecord>, CoordinationError>> {
        Box::pin(async move {
            let mut connection = self.manager.clone();
            // Redis time is the same clock used by the atomic heartbeat script.
            let time = self.finish(
                tokio::time::timeout(
                    self.command_timeout,
                    redis::cmd("TIME").query_async::<(i64, i64)>(&mut connection),
                )
                .await,
            )?;
            let now_ms = time.0.saturating_mul(1_000) + time.1 / 1_000;
            let ids = self.finish(
                tokio::time::timeout(
                    self.command_timeout,
                    redis::cmd("ZRANGEBYSCORE")
                        .arg(self.media_nodes_index())
                        .arg(format!("({now_ms}"))
                        .arg("+inf")
                        .arg("LIMIT")
                        .arg(0)
                        .arg(MAX_LIVE_NODES + 1)
                        .query_async::<Vec<String>>(&mut connection),
                )
                .await,
            )?;
            if ids.len() > usize::try_from(MAX_LIVE_NODES).unwrap_or(0) {
                return Err(CoordinationError::Unavailable(
                    "media node registry limit exceeded".into(),
                ));
            }
            if ids.is_empty() {
                return Ok(Vec::new());
            }
            let mut pipeline = redis::pipe();
            for id in &ids {
                pipeline
                    .cmd("HMGET")
                    .arg(self.media_node_key(id))
                    .arg("payload")
                    .arg("draining");
            }
            let fields = self.finish(
                tokio::time::timeout(
                    self.command_timeout,
                    pipeline.query_async::<Vec<(Option<Vec<u8>>, Option<String>)>>(&mut connection),
                )
                .await,
            )?;
            if fields.len() != ids.len() {
                return Err(CoordinationError::Unavailable(
                    "incomplete media registry read".into(),
                ));
            }
            let mut nodes = Vec::with_capacity(ids.len());
            for (id, fields) in ids.into_iter().zip(fields) {
                let Some(payload) = fields.0 else { continue };
                let mut node: MediaNodeRecord = serde_json::from_slice(&payload)
                    .map_err(|error| CoordinationError::Unexpected(error.to_string()))?;
                if node.node_id.as_str() != id {
                    return Err(CoordinationError::Unexpected(
                        "media node registry identity mismatch".into(),
                    ));
                }
                node.draining = fields.1.as_deref() == Some("1");
                nodes.push(node);
            }
            Ok(nodes)
        })
    }
}
