use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct ApiSettings {
    pub http_bind: String,
    pub operations_bind: String,
    pub media_grpc_url: String,
    pub allowed_origin: String,
    #[serde(default)]
    pub ws: WsSettings,
}

/// WebSocket gateway limits. Defaults keep the gateway bounded when a profile
/// does not override them.
#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct WsSettings {
    /// Maximum concurrent WebSocket connections across the process.
    pub max_connections: usize,
    /// Per-connection outbound queue depth.
    pub outbound_capacity: usize,
    /// Maximum time to flush one frame before the connection is closed.
    pub write_timeout_seconds: u64,
    /// Server ping cadence.
    pub heartbeat_interval_seconds: u64,
    /// Maximum quiet period before an idle connection is closed.
    pub heartbeat_timeout_seconds: u64,
    /// Window allowed for the client to authenticate after connecting.
    pub auth_timeout_seconds: u64,
    /// Maximum accepted frame/message size in bytes.
    pub max_message_bytes: usize,
}

impl Default for WsSettings {
    fn default() -> Self {
        Self {
            max_connections: 10_000,
            outbound_capacity: 64,
            write_timeout_seconds: 10,
            heartbeat_interval_seconds: 15,
            heartbeat_timeout_seconds: 45,
            auth_timeout_seconds: 10,
            max_message_bytes: 64 * 1024,
        }
    }
}
