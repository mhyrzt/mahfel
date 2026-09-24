use thiserror::Error;

#[derive(Debug, Error)]
pub enum MessagingError {
    #[error("AMQP operation failed: {0}")]
    Lapin(#[from] lapin::Error),
    #[error("job serialization failed: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("publish timed out")]
    Timeout,
    #[error("message could not be routed")]
    Unroutable,
    #[error("broker rejected the message")]
    Rejected,
    #[error("publisher confirms were not enabled")]
    NoConfirm,
}
