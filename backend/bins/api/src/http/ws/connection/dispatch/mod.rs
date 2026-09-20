pub(crate) mod helpers;
mod rtc;

pub use helpers::protocol_error;

use super::{CLOSE_PROTOCOL_ERROR, writer::Connection};
use crate::http::ws::{
    envelope::{self, ClientMessage, ProtocolError, ServerMessage},
    metrics::DisconnectReason,
    registry::ConnectionRegistry,
};
use mahfel_application::ports::Principal;
use mahfel_media_client::GrpcMediaGateway;
use uuid::Uuid;

pub(super) async fn handle_text(
    connection: &mut Connection,
    registry: &ConnectionRegistry,
    id: Uuid,
    text: &str,
    principal: &Principal,
    media: &GrpcMediaGateway,
) -> Result<(), DisconnectReason> {
    let envelope = match envelope::parse_envelope(text) {
        Ok(env) => env,
        Err(ProtocolError::UnsupportedVersion(v)) => {
            tracing::debug!(%v, "unsupported protocol version");
            connection
                .close(CLOSE_PROTOCOL_ERROR, "unsupported protocol version")
                .await;
            return Err(DisconnectReason::ProtocolError);
        }
        Err(ProtocolError::Malformed) => {
            return connection
                .send(protocol_error("malformed", "malformed message"))
                .await;
        }
    };
    let rid = envelope.id.clone();
    let kind = rtc_kind(&envelope.message);

    match envelope.message {
        ClientMessage::Echo { text } => connection.send(ServerMessage::Echo { text }).await,
        ClientMessage::Broadcast { text } => {
            registry.broadcast(&ServerMessage::Broadcast {
                from: id.to_string(),
                text,
            });
            Ok(())
        }
        ClientMessage::Auth { .. } => Ok(()),
        ClientMessage::CreateTransport { room_id } => {
            rtc::handle_create_transport(connection, rid, kind, &room_id, principal, media).await
        }
        ClientMessage::ConnectTransport {
            room_id,
            transport_id,
            dtls_parameters,
        } => {
            rtc::handle_connect_transport(
                connection,
                rid,
                kind,
                room_id,
                transport_id,
                dtls_parameters,
                media,
            )
            .await
        }
        ClientMessage::Produce {
            room_id,
            transport_id,
            kind: produce_kind,
            rtp_parameters,
        } => {
            rtc::handle_produce(
                connection,
                rid,
                kind,
                room_id,
                transport_id,
                produce_kind,
                rtp_parameters,
                principal,
                media,
            )
            .await
        }
        ClientMessage::Consume {
            room_id,
            transport_id,
            producer_id,
            rtp_capabilities,
        } => {
            rtc::handle_consume(
                connection,
                rid,
                kind,
                room_id,
                transport_id,
                producer_id,
                rtp_capabilities,
                principal,
                media,
            )
            .await
        }
        ClientMessage::PauseConsumer {
            room_id,
            consumer_id,
        } => rtc::handle_pause(connection, rid, kind, room_id, consumer_id, media, false).await,
        ClientMessage::ResumeConsumer {
            room_id,
            consumer_id,
        } => rtc::handle_pause(connection, rid, kind, room_id, consumer_id, media, true).await,
    }
}

fn rtc_kind(msg: &ClientMessage) -> &'static str {
    match msg {
        ClientMessage::CreateTransport { .. } => "create_transport",
        ClientMessage::ConnectTransport { .. } => "connect_transport",
        ClientMessage::Produce { .. } => "produce",
        ClientMessage::Consume { .. } => "consume",
        ClientMessage::PauseConsumer { .. } => "pause_consumer",
        ClientMessage::ResumeConsumer { .. } => "resume_consumer",
        _ => "",
    }
}
