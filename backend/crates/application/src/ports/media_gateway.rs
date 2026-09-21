use super::BoxFuture;
use mahfel_domain::RoomId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MediaGatewayError {
    #[error("media gateway failure: {0}")]
    Transport(String),
    #[error("media resource not found: {0}")]
    NotFound(String),
}

impl MediaGatewayError {
    pub fn transport<E: std::fmt::Display>(err: E) -> Self {
        Self::Transport(err.to_string())
    }
}

/// Media-plane control. Methods return [`BoxFuture`] so the port is
/// dyn-compatible.
pub trait MediaGateway: Send + Sync {
    fn create_room(&self, room: RoomId) -> BoxFuture<'_, Result<(), MediaGatewayError>>;

    fn close_room(&self, room: RoomId) -> BoxFuture<'_, Result<(), MediaGatewayError>>;
}
