use std::time::Duration;

/// Connection and namespacing settings for the Redis adapter.
///
/// The composition root maps configuration into this type; the adapter does not
/// depend on the configuration crate.
#[derive(Clone, Debug)]
pub struct RedisConnectionSettings {
    pub url: String,
    pub key_prefix: String,
    pub connect_timeout: Duration,
    pub command_timeout: Duration,
}
