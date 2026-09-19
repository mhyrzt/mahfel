use serde::Deserialize;

/// Ephemeral coordination backend (ADR 0006). Never authoritative.
#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct RedisSettings {
    pub url: String,
    /// Prefix applied to every key and channel family.
    pub key_prefix: String,
    pub connect_timeout_seconds: u64,
    pub command_timeout_seconds: u64,
}

impl Default for RedisSettings {
    fn default() -> Self {
        Self {
            url: "redis://redis:6379".to_owned(),
            key_prefix: "mahfel".to_owned(),
            connect_timeout_seconds: 5,
            command_timeout_seconds: 3,
        }
    }
}
