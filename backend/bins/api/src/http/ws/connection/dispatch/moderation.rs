//! Moderation frames: mute, unmute, remove, block, unblock.
//!
//! The transport adapter only. Capability resolution, the authoritative target
//! lookup, and the ordering of state change against media enforcement all live
//! in the application use cases, so REST and WebSocket decide identically.
//!
//! Fan-out resolves across all of a participant's connections, so a mute
//! reaches every device the target is on rather than only the newest one.

use super::super::writer::Connection;
use super::helpers::protocol_error;
use crate::http::ws::{
    envelope::ServerMessage, metrics::DisconnectReason, registry::ConnectionRegistry,
    state::WsState,
};
use mahfel_application::ApplicationError;
use mahfel_application::moderation::{BlockCommand, MuteCommand, MutedSource, RemoveCommand};
use mahfel_application::ports::Principal;
use mahfel_domain::{ActorId, BlockScope, MuteKind, ProducerSource, RoomId, SessionId};
use time::OffsetDateTime;
use uuid::Uuid;

/// Parse a session id, reporting a malformed one rather than guessing.
fn parse_session(raw: &str) -> Result<SessionId, &'static str> {
    Uuid::parse_str(raw)
        .map(SessionId::from_uuid)
        .map_err(|_| "invalid session_id")
}

fn parse_actor(raw: &str) -> Result<ActorId, &'static str> {
    Uuid::parse_str(raw)
        .map(ActorId::from_uuid)
        .map_err(|_| "invalid actor_id")
}

/// A mute kind from the wire. Unknown values are refused, not defaulted: the
/// difference decides whether the target can clear it.
fn parse_kind(raw: &str) -> Result<MuteKind, &'static str> {
    match raw.trim() {
        "temporary" => Ok(MuteKind::Temporary),
        "permanent" => Ok(MuteKind::Permanent),
        _ => Err("invalid kind"),
    }
}

/// A source from the wire, defaulting to the plain meaning of mute.
fn parse_source(raw: Option<&str>) -> Result<MutedSource, &'static str> {
    match raw.map(str::trim).filter(|raw| !raw.is_empty()) {
        None | Some("microphone") => Ok(MutedSource::Microphone),
        Some("camera") => Ok(MutedSource::Camera),
        Some("screen") => Ok(MutedSource::Screen),
        Some(_) => Err("invalid source"),
    }
}

/// Exactly one of a room or session target, for a block.
fn parse_scope(
    room_id: Option<String>,
    session_id: Option<String>,
) -> Result<BlockScope, &'static str> {
    match (room_id, session_id) {
        (Some(room_id), None) => Uuid::parse_str(&room_id)
            .map(|id| BlockScope::Room(RoomId::from_uuid(id)))
            .map_err(|_| "invalid room_id"),
        (None, Some(session_id)) => Uuid::parse_str(&session_id)
            .map(|id| BlockScope::Session(SessionId::from_uuid(id)))
            .map_err(|_| "invalid session_id"),
        _ => Err("exactly one of room_id or session_id is required"),
    }
}

fn source_name(source: ProducerSource) -> &'static str {
    source.as_str()
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn handle_mute(
    c: &mut Connection,
    registry: &ConnectionRegistry,
    rid: Option<String>,
    session_id: &str,
    actor_id: &str,
    kind: &str,
    source: Option<String>,
    principal: &Principal,
    state: &WsState,
) -> Result<(), DisconnectReason> {
    let session_id = match parse_session(session_id) {
        Ok(id) => id,
        Err(message) => {
            return c
                .send_with_id(rid, protocol_error("invalid_argument", message))
                .await;
        }
    };
    let subject = match parse_actor(actor_id) {
        Ok(id) => id,
        Err(message) => {
            return c
                .send_with_id(rid, protocol_error("invalid_argument", message))
                .await;
        }
    };
    let kind = match parse_kind(kind) {
        Ok(kind) => kind,
        Err(message) => {
            return c
                .send_with_id(rid, protocol_error("invalid_argument", message))
                .await;
        }
    };
    let source = match parse_source(source.as_deref()) {
        Ok(source) => source,
        Err(message) => {
            return c
                .send_with_id(rid, protocol_error("invalid_argument", message))
                .await;
        }
    };

    let outcome = match mahfel_application::moderation::mute(
        state.stores.participants.as_ref(),
        state.stores.sessions.as_ref(),
        state.stores.rooms.as_ref(),
        state.control_media(),
        MuteCommand {
            actor: principal.actor_id,
            session_id,
            subject,
            kind,
            source,
        },
        OffsetDateTime::now_utc(),
    )
    .await
    {
        Ok(outcome) => outcome,
        Err(error) => {
            let code = error.kind();
            let _ = c
                .send_with_id(None, protocol_error(error_code(code), &error.to_string()))
                .await;
            return Ok(());
        }
    };

    // The whole roster learns, and the target in particular must stop sending
    // media. Sent by actor id rather than connection, because moderation
    // resolves across every connection the target holds.
    registry.broadcast_to_session(
        session_id,
        &ServerMessage::ParticipantMuted {
            session_id: session_id.to_string(),
            participant_id: subject.to_string(),
            actor_id: subject.to_string(),
            kind: kind.as_str().to_owned(),
            source: source_name(outcome.source).to_owned(),
        },
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn handle_unmute(
    c: &mut Connection,
    registry: &ConnectionRegistry,
    rid: Option<String>,
    session_id: &str,
    actor_id: &str,
    source: Option<String>,
    principal: &Principal,
    state: &WsState,
) -> Result<(), DisconnectReason> {
    let session_id = match parse_session(session_id) {
        Ok(id) => id,
        Err(message) => {
            return c
                .send_with_id(rid, protocol_error("invalid_argument", message))
                .await;
        }
    };
    let subject = match parse_actor(actor_id) {
        Ok(id) => id,
        Err(message) => {
            return c
                .send_with_id(rid, protocol_error("invalid_argument", message))
                .await;
        }
    };
    let source = match parse_source(source.as_deref()) {
        Ok(source) => source,
        Err(message) => {
            return c
                .send_with_id(rid, protocol_error("invalid_argument", message))
                .await;
        }
    };

    // A self-unmute goes through the same use case, so the capability check is
    // identical whether the target or a moderator initiates it.
    let outcome = match mahfel_application::moderation::unmute(
        state.stores.participants.as_ref(),
        state.stores.sessions.as_ref(),
        state.stores.rooms.as_ref(),
        state.control_media(),
        MuteCommand {
            actor: principal.actor_id,
            session_id,
            subject,
            kind: MuteKind::Temporary,
            source,
        },
        OffsetDateTime::now_utc(),
    )
    .await
    {
        Ok(outcome) => outcome,
        Err(error) => {
            let code = error.kind();
            let _ = c
                .send_with_id(None, protocol_error(error_code(code), &error.to_string()))
                .await;
            return Ok(());
        }
    };

    registry.broadcast_to_session(
        session_id,
        &ServerMessage::ParticipantUnmuted {
            session_id: session_id.to_string(),
            participant_id: subject.to_string(),
            actor_id: subject.to_string(),
            source: source_name(outcome.source).to_owned(),
        },
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn handle_remove(
    c: &mut Connection,
    registry: &ConnectionRegistry,
    rid: Option<String>,
    session_id: &str,
    actor_id: &str,
    principal: &Principal,
    state: &WsState,
) -> Result<(), DisconnectReason> {
    let session_id = match parse_session(session_id) {
        Ok(id) => id,
        Err(message) => {
            return c
                .send_with_id(rid, protocol_error("invalid_argument", message))
                .await;
        }
    };
    let subject = match parse_actor(actor_id) {
        Ok(id) => id,
        Err(message) => {
            return c
                .send_with_id(rid, protocol_error("invalid_argument", message))
                .await;
        }
    };

    if let Err(error) = mahfel_application::moderation::remove_participant(
        state.stores.participants.as_ref(),
        state.stores.sessions.as_ref(),
        state.stores.rooms.as_ref(),
        state.control_media(),
        RemoveCommand {
            actor: principal.actor_id,
            session_id,
            subject,
        },
        OffsetDateTime::now_utc(),
    )
    .await
    {
        let code = error.kind();
        let _ = c
            .send_with_id(None, protocol_error(error_code(code), &error.to_string()))
            .await;
        return Ok(());
    }

    registry.broadcast_to_session(
        session_id,
        &ServerMessage::ParticipantRemoved {
            session_id: session_id.to_string(),
            participant_id: subject.to_string(),
            actor_id: subject.to_string(),
            reason: "removed".to_owned(),
        },
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn handle_block(
    c: &mut Connection,
    registry: &ConnectionRegistry,
    rid: Option<String>,
    room_id: Option<String>,
    session_id: Option<String>,
    actor_id: &str,
    reason: Option<String>,
    expires_at: Option<i64>,
    principal: &Principal,
    state: &WsState,
) -> Result<(), DisconnectReason> {
    let scope = match parse_scope(room_id, session_id) {
        Ok(scope) => scope,
        Err(message) => {
            return c
                .send_with_id(rid, protocol_error("invalid_argument", message))
                .await;
        }
    };
    let subject = match parse_actor(actor_id) {
        Ok(id) => id,
        Err(message) => {
            return c
                .send_with_id(rid, protocol_error("invalid_argument", message))
                .await;
        }
    };
    let expires_at =
        expires_at.and_then(|seconds| OffsetDateTime::from_unix_timestamp(seconds).ok());

    let outcome = match mahfel_application::moderation::block(
        state.stores.rooms.as_ref(),
        state.stores.sessions.as_ref(),
        state.stores.participants.as_ref(),
        state.stores.blocks.as_ref(),
        state.stores.rooms.as_ref(),
        state.control_media(),
        BlockCommand {
            actor: principal.actor_id,
            scope,
            subject,
            reason,
            expires_at,
        },
        OffsetDateTime::now_utc(),
    )
    .await
    {
        Ok(outcome) => outcome,
        Err(error) => {
            let code = error.kind();
            let _ = c
                .send_with_id(None, protocol_error(error_code(code), &error.to_string()))
                .await;
            return Ok(());
        }
    };

    let message = ServerMessage::PrincipalBlocked {
        room_id: scope.room_id().map(|id| id.to_string()),
        session_id: scope.session_id().map(|id| id.to_string()),
        actor_id: subject.to_string(),
    };
    match scope {
        // A room block concerns the room, so every session in it learns when
        // those sessions next run; today only the connections live on this
        // replica are reachable, which is the documented limitation.
        BlockScope::Room(room_id) => {
            let sessions = state
                .stores
                .sessions
                .list_for_room(room_id)
                .await
                .unwrap_or_default();
            for session in sessions {
                registry.broadcast_to_session(session.id, &message);
            }
        }
        BlockScope::Session(session_id) => registry.broadcast_to_session(session_id, &message),
    }
    tracing::info!(
        target: "audit",
        action = "moderation.block.frame",
        actor_id = %principal.actor_id,
        target_actor_id = %subject,
        created = outcome.created,
        ejected_from = outcome.ejected_from,
        outcome = "ok"
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn handle_unblock(
    c: &mut Connection,
    registry: &ConnectionRegistry,
    rid: Option<String>,
    room_id: Option<String>,
    session_id: Option<String>,
    actor_id: &str,
    principal: &Principal,
    state: &WsState,
) -> Result<(), DisconnectReason> {
    let scope = match parse_scope(room_id, session_id) {
        Ok(scope) => scope,
        Err(message) => {
            return c
                .send_with_id(rid, protocol_error("invalid_argument", message))
                .await;
        }
    };
    let subject = match parse_actor(actor_id) {
        Ok(id) => id,
        Err(message) => {
            return c
                .send_with_id(rid, protocol_error("invalid_argument", message))
                .await;
        }
    };

    if let Err(error) = mahfel_application::moderation::unblock(
        state.stores.rooms.as_ref(),
        state.stores.sessions.as_ref(),
        state.stores.blocks.as_ref(),
        state.stores.rooms.as_ref(),
        BlockCommand {
            actor: principal.actor_id,
            scope,
            subject,
            reason: None,
            expires_at: None,
        },
        OffsetDateTime::now_utc(),
    )
    .await
    {
        let code = error.kind();
        let _ = c
            .send_with_id(None, protocol_error(error_code(code), &error.to_string()))
            .await;
        return Ok(());
    }

    let message = ServerMessage::PrincipalUnblocked {
        room_id: scope.room_id().map(|id| id.to_string()),
        session_id: scope.session_id().map(|id| id.to_string()),
        actor_id: subject.to_string(),
    };
    match scope {
        BlockScope::Room(room_id) => {
            let sessions = state
                .stores
                .sessions
                .list_for_room(room_id)
                .await
                .unwrap_or_default();
            for session in sessions {
                registry.broadcast_to_session(session.id, &message);
            }
        }
        BlockScope::Session(session_id) => registry.broadcast_to_session(session_id, &message),
    }
    Ok(())
}

/// A stable, machine-readable code for a moderation failure.
///
/// Deliberately coarse: a caller learns the category, not which participant
/// exists, so the frame cannot be used to enumerate a roster.
pub(super) fn error_code(kind: mahfel_application::error::ErrorKind) -> &'static str {
    use mahfel_application::error::ErrorKind;
    match kind {
        ErrorKind::NotFound => "not_found",
        ErrorKind::Forbidden => "forbidden",
        ErrorKind::Conflict => "conflict",
        ErrorKind::Unprocessable => "invalid_argument",
        ErrorKind::Unauthorized => "unauthorized",
        _ => "internal",
    }
}
