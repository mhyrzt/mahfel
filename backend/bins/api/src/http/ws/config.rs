use mahfel_config::WsSettings;
use std::time::Duration;

/// Bounded limits for the WebSocket gateway.
///
/// Every value is explicit so concurrency, queueing, and timeout behavior are
/// reviewable and tunable per environment (and shrinkable in tests).
#[derive(Clone, Debug)]
pub struct WsConfig {
    pub max_connections: usize,
    pub outbound_capacity: usize,
    pub write_timeout: Duration,
    pub heartbeat_interval: Duration,
    pub heartbeat_timeout: Duration,
    pub auth_timeout: Duration,
    pub max_message_bytes: usize,
}

impl Default for WsConfig {
    fn default() -> Self {
        Self {
            max_connections: 10_000,
            outbound_capacity: 64,
            write_timeout: Duration::from_secs(10),
            heartbeat_interval: Duration::from_secs(15),
            heartbeat_timeout: Duration::from_secs(45),
            auth_timeout: Duration::from_secs(10),
            max_message_bytes: 64 * 1024,
        }
    }
}

impl From<&WsSettings> for WsConfig {
    fn from(settings: &WsSettings) -> Self {
        Self {
            max_connections: settings.max_connections,
            outbound_capacity: settings.outbound_capacity,
            write_timeout: Duration::from_secs(settings.write_timeout_seconds),
            heartbeat_interval: Duration::from_secs(settings.heartbeat_interval_seconds),
            heartbeat_timeout: Duration::from_secs(settings.heartbeat_timeout_seconds),
            auth_timeout: Duration::from_secs(settings.auth_timeout_seconds),
            max_message_bytes: settings.max_message_bytes,
        }
    }
}
