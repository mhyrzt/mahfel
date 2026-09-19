use super::{CLOSE_PROTOCOL_ERROR, writer::Connection};
use crate::http::ws::{
    envelope::{self, ClientMessage, ProtocolError, ServerMessage},
    metrics::DisconnectReason,
    registry::ConnectionRegistry,
};
use uuid::Uuid;

/// Apply one client text frame. `Err(reason)` requests a disconnect.
pub(super) async fn handle_text(
    connection: &mut Connection,
    registry: &ConnectionRegistry,
    id: Uuid,
    text: &str,
) -> Result<(), DisconnectReason> {
    match envelope::parse(text) {
        Ok(ClientMessage::Echo { text }) => connection.send(ServerMessage::Echo { text }).await,
        Ok(ClientMessage::Broadcast { text }) => {
            registry.broadcast(&ServerMessage::Broadcast {
                from: id.to_string(),
                text,
            });
            Ok(())
        }
        // Already authenticated; ignore a repeated auth frame.
        Ok(ClientMessage::Auth { .. }) => Ok(()),
        Err(ProtocolError::UnsupportedVersion(version)) => {
            tracing::debug!(%version, "websocket closed: unsupported protocol version");
            connection
                .close(CLOSE_PROTOCOL_ERROR, "unsupported protocol version")
                .await;
            Err(DisconnectReason::ProtocolError)
        }
        Err(ProtocolError::Malformed) => {
            connection
                .send(protocol_error("malformed", "malformed message"))
                .await
        }
    }
}

/// Build a server error frame.
pub(super) fn protocol_error(code: &str, message: &str) -> ServerMessage {
    ServerMessage::Error {
        code: code.to_owned(),
        message: message.to_owned(),
    }
}
