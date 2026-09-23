use crate::{RedisCoordination, RedisError};
use mahfel_domain::{ParticipantId, SessionId};
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

const MAX_CONNECTIONS_PER_SESSION: usize = 2_000;
const MUTATE: &str = r#"
local now = redis.call('TIME')
local millis = now[1] * 1000 + math.floor(now[2] / 1000)
local expired = redis.call('ZRANGEBYSCORE', KEYS[1], '-inf', millis)
if #expired > 0 then
  redis.call('ZREM', KEYS[1], unpack(expired))
  redis.call('HDEL', KEYS[2], unpack(expired))
end
local changed = #expired > 0
if ARGV[1] == 'join' then
  if redis.call('HLEN', KEYS[2]) >= tonumber(ARGV[4]) and
     redis.call('HEXISTS', KEYS[2], ARGV[2]) == 0 then
    return redis.error_reply('session presence capacity exceeded')
  end
  if redis.call('HGET', KEYS[2], ARGV[2]) ~= ARGV[3] then changed = true end
  redis.call('HSET', KEYS[2], ARGV[2], ARGV[3])
  redis.call('ZADD', KEYS[1], millis + tonumber(ARGV[5]), ARGV[2])
elseif ARGV[1] == 'leave' then
  if redis.call('HDEL', KEYS[2], ARGV[2]) > 0 then changed = true end
  redis.call('ZREM', KEYS[1], ARGV[2])
end
local version = tonumber(redis.call('GET', KEYS[3]) or '0')
if changed then
  version = redis.call('INCR', KEYS[3])
  redis.call('PUBLISH', KEYS[4], 'wake')
end
local retention = tonumber(ARGV[5]) * 2
redis.call('PEXPIRE', KEYS[1], retention)
redis.call('PEXPIRE', KEYS[2], retention)
redis.call('PEXPIRE', KEYS[3], retention)
local remaining = 0
for _, participant in ipairs(redis.call('HVALS', KEYS[2])) do
  if participant == ARGV[3] then remaining = remaining + 1 end
end
return {version, remaining}
"#;

const RENEW: &str = r#"
if redis.call('HGET', KEYS[2], ARGV[1]) ~= ARGV[2] then return 0 end
local now = redis.call('TIME')
local millis = now[1] * 1000 + math.floor(now[2] / 1000)
redis.call('ZADD', KEYS[1], millis + tonumber(ARGV[3]), ARGV[1])
redis.call('PEXPIRE', KEYS[1], tonumber(ARGV[3]) * 2)
redis.call('PEXPIRE', KEYS[2], tonumber(ARGV[3]) * 2)
redis.call('PEXPIRE', KEYS[3], tonumber(ARGV[3]) * 2)
return 1
"#;

const SNAPSHOT: &str = r#"
local now = redis.call('TIME')
local millis = now[1] * 1000 + math.floor(now[2] / 1000)
local expired = redis.call('ZRANGEBYSCORE', KEYS[1], '-inf', millis)
if #expired > 0 then
  redis.call('ZREM', KEYS[1], unpack(expired))
  redis.call('HDEL', KEYS[2], unpack(expired))
  redis.call('INCR', KEYS[3])
  redis.call('PUBLISH', KEYS[4], 'wake')
end
local values = redis.call('HVALS', KEYS[2])
if #values > tonumber(ARGV[1]) then
  return redis.error_reply('session presence capacity exceeded')
end
local result = {tonumber(redis.call('GET', KEYS[3]) or '0')}
for _, value in ipairs(values) do table.insert(result, value) end
return result
"#;

/// The lease set is ephemeral. PostgreSQL still owns participant identity and
/// connected-time history; this view only describes currently live sockets.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionPresenceSnapshot {
    pub version: u64,
    pub participant_ids: Vec<ParticipantId>,
}

impl RedisCoordination {
    fn session_keys(&self, session_id: SessionId) -> [String; 4] {
        let session = session_id.to_string();
        [
            self.keys.key("session-leases", &session),
            self.keys.key("session-lease-participants", &session),
            self.keys.key("session-presence-version", &session),
            self.keys.channel("session-presence"),
        ]
    }

    pub async fn session_presence_join(
        &self,
        session_id: SessionId,
        connection_id: Uuid,
        participant_id: ParticipantId,
        ttl: Duration,
    ) -> Result<u64, RedisError> {
        self.mutate_session_presence("join", session_id, connection_id, participant_id, ttl)
            .await
    }

    pub async fn session_presence_leave(
        &self,
        session_id: SessionId,
        connection_id: Uuid,
        participant_id: ParticipantId,
        ttl: Duration,
    ) -> Result<u64, RedisError> {
        self.mutate_session_presence("leave", session_id, connection_id, participant_id, ttl)
            .await
    }

    async fn mutate_session_presence(
        &self,
        action: &str,
        session_id: SessionId,
        connection_id: Uuid,
        participant_id: ParticipantId,
        ttl: Duration,
    ) -> Result<u64, RedisError> {
        let keys = self.session_keys(session_id);
        let mut connection = self.manager.clone();
        let ttl_ms = u64::try_from(ttl.as_millis()).unwrap_or(u64::MAX);
        let result = timeout(
            self.command_timeout,
            redis::cmd("EVAL")
                .arg(MUTATE)
                .arg(keys.len())
                .arg(&keys[..])
                .arg(action)
                .arg(connection_id.to_string())
                .arg(participant_id.to_string())
                .arg(MAX_CONNECTIONS_PER_SESSION)
                .arg(ttl_ms)
                .query_async::<(u64, u64)>(&mut connection),
        )
        .await;
        self.finish(result).map(|(_, remaining)| remaining)
    }

    /// Refresh only an existing lease. A delayed heartbeat cannot recreate a
    /// connection that has already left.
    pub async fn session_presence_renew(
        &self,
        session_id: SessionId,
        connection_id: Uuid,
        participant_id: ParticipantId,
        ttl: Duration,
    ) -> Result<bool, RedisError> {
        let keys = self.session_keys(session_id);
        let mut connection = self.manager.clone();
        let ttl_ms = u64::try_from(ttl.as_millis()).unwrap_or(u64::MAX);
        let result = timeout(
            self.command_timeout,
            redis::cmd("EVAL")
                .arg(RENEW)
                .arg(3)
                .arg(&keys[..3])
                .arg(connection_id.to_string())
                .arg(participant_id.to_string())
                .arg(ttl_ms)
                .query_async::<u64>(&mut connection),
        )
        .await;
        self.finish(result).map(|renewed| renewed != 0)
    }

    pub async fn session_presence_snapshot(
        &self,
        session_id: SessionId,
    ) -> Result<SessionPresenceSnapshot, RedisError> {
        let keys = self.session_keys(session_id);
        let mut connection = self.manager.clone();
        let result = timeout(
            self.command_timeout,
            redis::cmd("EVAL")
                .arg(SNAPSHOT)
                .arg(keys.len())
                .arg(&keys[..])
                .arg(MAX_CONNECTIONS_PER_SESSION)
                .query_async::<Vec<String>>(&mut connection),
        )
        .await;
        let values = self.finish(result)?;
        let Some((first, participants)) = values.split_first() else {
            return Err(RedisError::Unavailable(
                "empty session presence result".to_owned(),
            ));
        };
        let version = first.parse().map_err(|error| {
            RedisError::Unavailable(format!("invalid presence version: {error}"))
        })?;
        let mut participant_ids = participants
            .iter()
            .map(|value| {
                let id = Uuid::parse_str(value)
                    .map_err(|error| RedisError::Unavailable(error.to_string()))?;
                Ok(ParticipantId::from_uuid(id))
            })
            .collect::<Result<Vec<_>, RedisError>>()?;
        participant_ids.sort_unstable_by_key(|id| id.as_uuid());
        participant_ids.dedup();
        Ok(SessionPresenceSnapshot {
            version,
            participant_ids,
        })
    }
}
