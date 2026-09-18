pub mod event_publisher;
pub mod media_gateway;
pub mod room_store;
pub mod session_store;
pub mod user_store;

pub use event_publisher::{DomainEvent, EventPublisher, PublishError};
pub use media_gateway::{MediaGateway, MediaGatewayError};
pub use room_store::RoomStore;
pub use session_store::SessionStore;
pub use user_store::UserStore;

use thiserror::Error;

/// Generic storage failure for application ports.
///
/// Infrastructure adapters map their concrete errors (`sqlx::Error`,
/// `DatabaseError`, etc.) into this type so `mahfel-application`
/// never depends on `sqlx`/`lapin`/`tonic` directly.
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("storage failure: {0}")]
    Unexpected(String),
    #[error("entity already exists: {0}")]
    Conflict(String),
    #[error("entity not found: {0}")]
    NotFound(String),
}

impl StoreError {
    pub fn unexpected<E: std::fmt::Display>(err: E) -> Self {
        Self::Unexpected(err.to_string())
    }
}
