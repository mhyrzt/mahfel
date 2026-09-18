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

pub trait MediaGateway: Send + Sync {
    fn create_room(
        &self,
        room: RoomId,
    ) -> impl std::future::Future<Output = Result<(), MediaGatewayError>> + Send;
    fn close_room(
        &self,
        room: RoomId,
    ) -> impl std::future::Future<Output = Result<(), MediaGatewayError>> + Send;
}
