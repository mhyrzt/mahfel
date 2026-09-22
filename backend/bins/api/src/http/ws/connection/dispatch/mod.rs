pub(crate) mod helpers;
mod moderation;
mod participant;
mod rtc;

pub use helpers::protocol_error;

use super::{CLOSE_PROTOCOL_ERROR, writer::Connection};
use crate::http::ws::{
    envelope::{self, ClientMessage, ProtocolError, ServerMessage},
    metrics::DisconnectReason,
    registry::ConnectionRegistry,
    state::WsState,
};
use mahfel_application::ports::Principal;
use uuid::Uuid;

pub(super) async fn handle_text(
    connection: &mut Connection,
    registry: &ConnectionRegistry,
    id: Uuid,
    text: &str,
    principal: &Principal,
    state: &WsState,
) -> Result<(), DisconnectReason> {
    let media = state.media.as_ref();
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
        ClientMessage::Join { session_id } => {
            participant::handle_join(connection, registry, id, &session_id, principal, state).await
        }
        ClientMessage::Leave {} => {
            participant::handle_leave(connection, registry, id, principal, state).await
        }
        ClientMessage::MuteParticipant {
            session_id,
            actor_id,
            kind: mute_kind,
            source,
        } => {
            moderation::handle_mute(
                connection,
                registry,
                rid,
                &session_id,
                &actor_id,
                &mute_kind,
                source,
                principal,
                state,
            )
            .await
        }
        ClientMessage::UnmuteParticipant {
            session_id,
            actor_id,
            source,
        } => {
            moderation::handle_unmute(
                connection,
                registry,
                rid,
                &session_id,
                &actor_id,
                source,
                principal,
                state,
            )
            .await
        }
        ClientMessage::RemoveParticipant {
            session_id,
            actor_id,
        } => {
            moderation::handle_remove(
                connection,
                registry,
                rid,
                &session_id,
                &actor_id,
                principal,
                state,
            )
            .await
        }
        ClientMessage::BlockPrincipal {
            room_id,
            session_id,
            actor_id,
            reason,
            expires_at,
        } => {
            moderation::handle_block(
                connection, registry, rid, room_id, session_id, &actor_id, reason, expires_at,
                principal, state,
            )
            .await
        }
        ClientMessage::UnblockPrincipal {
            room_id,
            session_id,
            actor_id,
        } => {
            moderation::handle_unblock(
                connection, registry, rid, room_id, session_id, &actor_id, principal, state,
            )
            .await
        }
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
            source,
        } => {
            rtc::handle_produce(
                connection,
                rid,
                kind,
                room_id,
                transport_id,
                produce_kind,
                rtp_parameters,
                source,
                principal,
                state,
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
