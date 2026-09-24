use lapin::{Connection, ConnectionProperties};

use super::MessagingError;

pub async fn connect(amqp_url: &str) -> Result<Connection, MessagingError> {
    Ok(Connection::connect(amqp_url, ConnectionProperties::default()).await?)
}
