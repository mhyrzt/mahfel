use crate::{
    error::RedisError, keys::Keys, metrics::RedisMetrics, settings::RedisConnectionSettings,
};
use futures_util::StreamExt;
use mahfel_application::ports::{BoxFuture, CoordinationError, CoordinationStore};
use redis::aio::ConnectionManager;
use std::time::Duration;
use tokio::{sync::mpsc, time::timeout};

/// Bounded queue depth for a subscription.
const SUBSCRIBER_CAPACITY: usize = 256;
const FIXED_WINDOW_SCRIPT: &str = "\
local count = redis.call('INCR', KEYS[1]); \
if count == 1 then redis.call('PEXPIRE', KEYS[1], ARGV[1]); end; \
return count";

/// A message received from a pub/sub channel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RedisMessage {
    pub channel: String,
    pub payload: Vec<u8>,
}

/// Redis-backed ephemeral coordination.
///
/// The command path uses a multiplexed, auto-reconnecting [`ConnectionManager`];
/// pub/sub uses a dedicated connection and forwards messages through a bounded
/// queue.
#[derive(Clone)]
pub struct RedisCoordination {
    url: String,
    pub(crate) manager: ConnectionManager,
    pub(crate) keys: Keys,
    pub(crate) command_timeout: Duration,
    metrics: RedisMetrics,
}

impl RedisCoordination {
    pub async fn connect(
        settings: &RedisConnectionSettings,
        metrics: RedisMetrics,
    ) -> Result<Self, RedisError> {
        let client = redis::Client::open(settings.url.as_str())?;
        let manager = timeout(settings.connect_timeout, client.get_connection_manager())
            .await
            .map_err(|_| RedisError::Unavailable("connect timed out".to_owned()))??;
        Ok(Self {
            url: settings.url.clone(),
            manager,
            keys: Keys::new(settings.key_prefix.clone()),
            command_timeout: settings.command_timeout,
            metrics,
        })
    }

    /// Namespaced key and channel builders.
    #[must_use]
    pub fn keys(&self) -> &Keys {
        &self.keys
    }

    /// Liveness probe for readiness reporting.
    pub async fn ping(&self) -> Result<(), RedisError> {
        let mut connection = self.manager.clone();
        let result = timeout(
            self.command_timeout,
            redis::cmd("PING").query_async::<String>(&mut connection),
        )
        .await;
        self.finish(result).map(|_| ())
    }

    /// Subscribe to a channel. Messages arrive on a bounded queue; a slow
    /// consumer stops the forwarder rather than growing without limit.
    pub async fn subscribe(
        &self,
        channel: &str,
    ) -> Result<mpsc::Receiver<RedisMessage>, RedisError> {
        let client = redis::Client::open(self.url.as_str())?;
        let mut pubsub = client.get_async_pubsub().await?;
        pubsub.subscribe(channel).await?;

        let (sender, receiver) = mpsc::channel(SUBSCRIBER_CAPACITY);
        let metrics = self.metrics.clone();
        tokio::spawn(async move {
            let mut stream = pubsub.on_message();
            while let Some(message) = stream.next().await {
                metrics.message_received();
                let received = RedisMessage {
                    channel: message.get_channel_name().to_owned(),
                    payload: message.get_payload_bytes().to_vec(),
                };
                if sender.try_send(received).is_err() {
                    break;
                }
            }
        });
        Ok(receiver)
    }

    pub(crate) fn finish<T>(
        &self,
        result: Result<Result<T, redis::RedisError>, tokio::time::error::Elapsed>,
    ) -> Result<T, RedisError> {
        match result {
            Ok(Ok(value)) => {
                self.metrics.command_ok();
                Ok(value)
            }
            Ok(Err(error)) => {
                self.metrics.command_error();
                Err(RedisError::from(error))
            }
            Err(_) => {
                self.metrics.command_error();
                Err(RedisError::Unavailable("command timed out".to_owned()))
            }
        }
    }
}

impl CoordinationStore for RedisCoordination {
    fn set_with_ttl<'a>(
        &'a self,
        key: &'a str,
        value: &'a [u8],
        ttl: Duration,
    ) -> BoxFuture<'a, Result<(), CoordinationError>> {
        Box::pin(async move {
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
            self.finish(result)
                .map(|_| ())
                .map_err(CoordinationError::from)
        })
    }

    fn get<'a>(
        &'a self,
        key: &'a str,
    ) -> BoxFuture<'a, Result<Option<Vec<u8>>, CoordinationError>> {
        Box::pin(async move {
            let mut connection = self.manager.clone();
            let result = timeout(
                self.command_timeout,
                redis::cmd("GET")
                    .arg(key)
                    .query_async::<Option<Vec<u8>>>(&mut connection),
            )
            .await;
            self.finish(result).map_err(CoordinationError::from)
        })
    }

    fn delete<'a>(&'a self, key: &'a str) -> BoxFuture<'a, Result<(), CoordinationError>> {
        Box::pin(async move {
            let mut connection = self.manager.clone();
            let result = timeout(
                self.command_timeout,
                redis::cmd("DEL")
                    .arg(key)
                    .query_async::<i64>(&mut connection),
            )
            .await;
            self.finish(result)
                .map(|_| ())
                .map_err(CoordinationError::from)
        })
    }

    fn publish<'a>(
        &'a self,
        channel: &'a str,
        payload: &'a [u8],
    ) -> BoxFuture<'a, Result<(), CoordinationError>> {
        Box::pin(async move {
            let mut connection = self.manager.clone();
            let result = timeout(
                self.command_timeout,
                redis::cmd("PUBLISH")
                    .arg(channel)
                    .arg(payload)
                    .query_async::<i64>(&mut connection),
            )
            .await;
            let outcome = self.finish(result);
            if outcome.is_ok() {
                self.metrics.message_published();
            }
            outcome.map(|_| ()).map_err(CoordinationError::from)
        })
    }

    fn increment_with_ttl<'a>(
        &'a self,
        key: &'a str,
        ttl: Duration,
    ) -> BoxFuture<'a, Result<u64, CoordinationError>> {
        Box::pin(async move {
            let mut connection = self.manager.clone();
            let milliseconds = u64::try_from(ttl.as_millis()).unwrap_or(u64::MAX);
            let namespaced = self.keys.key("counter", key);
            let result = timeout(
                self.command_timeout,
                redis::cmd("EVAL")
                    .arg(FIXED_WINDOW_SCRIPT)
                    .arg(1)
                    .arg(&namespaced)
                    .arg(milliseconds)
                    .query_async::<i64>(&mut connection),
            )
            .await;
            let count = self.finish(result).map_err(CoordinationError::from)?;
            Ok(u64::try_from(count).unwrap_or(0))
        })
    }
}
