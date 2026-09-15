use lapin::{Connection, ConnectionProperties};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MessagingError {
    #[error("AMQP connection failed: {0}")]
    Lapin(#[from] lapin::Error),
}

pub async fn connect(amqp_url: &str) -> Result<Connection, MessagingError> {
    Connection::connect(amqp_url, ConnectionProperties::default())
        .await
        .map_err(MessagingError::from)
}
