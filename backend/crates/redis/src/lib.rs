//! Redis coordination adapter (ADR 0006).
//!
//! Ephemeral state only: namespaced keys with TTLs and pub/sub fan-out. An
//! outage surfaces as [`CoordinationError::Unavailable`] so callers degrade;
//! the connection manager reconnects in the background.

mod client;
mod error;
mod keys;
mod media_registry;
mod metrics;
mod settings;

pub use client::{RedisCoordination, RedisMessage};
pub use error::RedisError;
pub use keys::Keys;
pub use metrics::RedisMetrics;
pub use settings::RedisConnectionSettings;
