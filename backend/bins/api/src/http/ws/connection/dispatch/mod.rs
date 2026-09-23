pub(crate) mod helpers;
mod moderation;
mod participant;
mod rtc;

pub use helpers::protocol_error;
pub(super) use participant::close_owned_transports;

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
    let rtc_participation = if let Some(raw_room) = rtc_room_id(&envelope.message) {
        let Some(handle) = registry.participation(id) else {
            return connection
                .send_with_id(rid, protocol_error("not_joined", "join a session first"))
                .await;
        };
        if helpers::parse_room_id(raw_room).ok() != Some(handle.room_id) {
            return connection
                .send_with_id(
                    rid,
                    protocol_error("forbidden", "room does not match joined session"),
                )
                .await;
        }
        Some(handle)
    } else {
        None
    };
    let media = if let Some(handle) = rtc_participation {
        match state.media_for(handle.session_id, handle.room_id).await {
            Ok(media) => media,
            Err(_) => {
                return connection
                    .send_with_id(
                        rid,
                        protocol_error("unavailable", "assigned media node unavailable"),
                    )
                    .await;
            }
        }
    } else {
        std::sync::Arc::clone(&state.media)
    };
    let media_room = state.placement.as_ref().and_then(|_| {
        rtc_participation.map(|handle| crate::http::ws::placement::media_room_id(handle.session_id))
    });

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
            rtc::handle_create_transport(
                connection,
                rid,
                kind,
                &room_id,
                principal,
                rtc_participation.map(|h| h.session_id),
                media_room,
                media.as_ref(),
            )
            .await
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
                media_room,
                media.as_ref(),
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
                media_room,
                media.as_ref(),
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
                media_room,
                media.as_ref(),
            )
            .await
        }
        ClientMessage::PauseConsumer {
            room_id,
            consumer_id,
        } => {
            rtc::handle_pause(
                connection,
                rid,
                kind,
                room_id,
                consumer_id,
                media_room,
                media.as_ref(),
                false,
            )
            .await
        }
        ClientMessage::ResumeConsumer {
            room_id,
            consumer_id,
        } => {
            rtc::handle_pause(
                connection,
                rid,
                kind,
                room_id,
                consumer_id,
                media_room,
                media.as_ref(),
                true,
            )
            .await
        }
    }
}

fn rtc_room_id(message: &ClientMessage) -> Option<&str> {
    match message {
        ClientMessage::CreateTransport { room_id }
        | ClientMessage::ConnectTransport { room_id, .. }
        | ClientMessage::Produce { room_id, .. }
        | ClientMessage::Consume { room_id, .. }
        | ClientMessage::PauseConsumer { room_id, .. }
        | ClientMessage::ResumeConsumer { room_id, .. } => Some(room_id),
        _ => None,
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
