use crate::{RedisCoordination, RedisError};
use std::time::Duration;
use tokio::time::timeout;

const CONSUME: &str = "local value = redis.call('GET', KEYS[1]); if value then redis.call('DEL', KEYS[1]); end; return value";

impl RedisCoordination {
    /// Store a namespaced short-lived value for atomic single-use retrieval.
    pub async fn store_one_time(
        &self,
        family: &str,
        id: &str,
        value: &[u8],
        ttl: Duration,
    ) -> Result<(), RedisError> {
        let key = self.keys.key(family, id);
        let mut connection = self.manager.clone();
        let milliseconds = u64::try_from(ttl.as_millis()).unwrap_or(u64::MAX);
        let result = timeout(
            self.command_timeout,
            redis::cmd("SET")
                .arg(key)
                .arg(value)
                .arg("PX")
                .arg(milliseconds)
                .query_async::<String>(&mut connection),
        )
        .await;
        self.finish(result).map(|_| ())
    }

    /// Atomically retrieve and delete a one-time value.
    pub async fn consume_one_time(
        &self,
        family: &str,
        id: &str,
    ) -> Result<Option<Vec<u8>>, RedisError> {
        let key = self.keys.key(family, id);
        let mut connection = self.manager.clone();
        let result = timeout(
            self.command_timeout,
            redis::Script::new(CONSUME)
                .key(key)
                .invoke_async::<Option<Vec<u8>>>(&mut connection),
        )
        .await;
        self.finish(result)
    }
}
