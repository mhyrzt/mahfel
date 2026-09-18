use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DomainEvent {
    pub id: Uuid,
    pub kind: String,
    pub payload: serde_json::Value,
}

impl DomainEvent {
    pub fn new(kind: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind: kind.into(),
            payload,
        }
    }
}

#[derive(Debug, Error)]
pub enum PublishError {
    #[error("event publish failed: {0}")]
    Transport(String),
}

impl PublishError {
    pub fn transport<E: std::fmt::Display>(err: E) -> Self {
        Self::Transport(err.to_string())
    }
}

pub trait EventPublisher: Send + Sync {
    fn publish(
        &self,
        event: DomainEvent,
    ) -> impl std::future::Future<Output = Result<(), PublishError>> + Send;
}
