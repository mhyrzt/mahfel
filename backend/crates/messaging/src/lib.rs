use lapin::{BasicProperties, Connection, ConnectionProperties, options::BasicPublishOptions};
use mahfel_application::ports::{BoxFuture, DomainEvent, EventPublisher, PublishError};
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

/// Adapter that hides `lapin::Connection` behind the application `EventPublisher` port.
pub struct RabbitMqEventPublisher {
    connection: Connection,
}

impl RabbitMqEventPublisher {
    #[must_use]
    pub fn new(connection: Connection) -> Self {
        Self { connection }
    }

    #[must_use]
    pub fn into_inner(self) -> Connection {
        self.connection
    }
}

impl EventPublisher for RabbitMqEventPublisher {
    fn publish(&self, event: DomainEvent) -> BoxFuture<'_, Result<(), PublishError>> {
        Box::pin(async move {
            let channel = self
                .connection
                .create_channel()
                .await
                .map_err(PublishError::transport)?;

            let payload = serde_json::to_vec(&event).map_err(PublishError::transport)?;

            channel
                .basic_publish(
                    "mahfel.events".into(),
                    event.kind.as_str().into(),
                    BasicPublishOptions::default(),
                    &payload,
                    BasicProperties::default(),
                )
                .await
                .map_err(PublishError::transport)?
                .await
                .map_err(PublishError::transport)?;

            Ok(())
        })
    }
}
