//! Session participation frames.
//!
//! The durable participant identity and the join authorization live in the
//! application layer; this module is the transport adapter. It resolves the
//! stable participant id, records it on the connection, and propagates roster
//! changes to the other connections in the session.

use super::super::writer::Connection;
use crate::http::{
    ws::state::WsState,
    ws::{envelope::ServerMessage, metrics::DisconnectReason, registry::ConnectionRegistry},
};
use mahfel_application::{
    ports::Principal,
    sessions::{
        JoinOutcome, JoinSession, LeaveSession, SessionOperationError, join_session, leave_session,
        preflight_join,
    },
};
use mahfel_domain::AdmissionOrigin;
use mahfel_domain::SessionId;
use uuid::Uuid;

fn parse_session_id(raw: &str) -> Result<SessionId, ServerMessage> {
    Uuid::parse_str(raw)
        .map(SessionId::from_uuid)
        .map_err(|_| ServerMessage::JoinRejected {
            code: "invalid_session_id".to_owned(),
            message: "session_id must be a uuid".to_owned(),
        })
}

/// Handle a `join` frame: authorize, resolve identity, and announce.
pub(super) async fn handle_join(
    connection: &mut Connection,
    registry: &ConnectionRegistry,
    id: Uuid,
    raw_session_id: &str,
    principal: &Principal,
    state: &WsState,
) -> Result<(), DisconnectReason> {
    let session_id = match parse_session_id(raw_session_id) {
        Ok(session_id) => session_id,
        Err(rejection) => return connection.send(rejection).await,
    };
    let lifecycle_lock = registry.lifecycle_lock(session_id, principal.actor_id);
    let lifecycle_guard = lifecycle_lock.lock().await;

    let command = JoinSession {
        actor: principal.actor_id,
        session_id,
        origin: AdmissionOrigin::Join,
    };
    // Placement must be ready before the durable participant and connected
    // interval are written. Lobby joins allocate no media capacity.
    if let Some(router) = &state.placement {
        let room_id = match preflight_join(
            state.stores.participants.as_ref(),
            state.stores.sessions.as_ref(),
            state.stores.rooms.as_ref(),
            state.stores.blocks.as_ref(),
            &command,
        )
        .await
        {
            Ok(room_id) => room_id,
            Err(error) => {
                return connection
                    .send(ServerMessage::JoinRejected {
                        code: join_rejection_code(&error).to_owned(),
                        message: "join rejected".to_owned(),
                    })
                    .await;
            }
        };
        if let Some(room_id) = room_id
            && router.place(session_id, room_id).await.is_err()
        {
            return connection
                .send(ServerMessage::JoinRejected {
                    code: "unavailable".to_owned(),
                    message: "media placement unavailable".to_owned(),
                })
                .await;
        }
    }

    let (participant, room_id) = match join_session(
        state.stores.participants.as_ref(),
        state.stores.participation.as_ref(),
        state.stores.admissions.as_ref(),
        state.stores.sessions.as_ref(),
        state.stores.rooms.as_ref(),
        state.stores.blocks.as_ref(),
        command,
    )
    .await
    {
        Ok(JoinOutcome::Joined {
            participant,
            room_id,
        }) => (participant, room_id),
        // Authorized but not yet admitted: the caller is waiting, so it holds no
        // participant identity and can attach to no session resources.
        Ok(JoinOutcome::Waiting(request)) => {
            return connection
                .send(ServerMessage::JoinWaiting {
                    session_id: session_id.to_string(),
                    request_id: request.id.to_string(),
                })
                .await;
        }
        Err(error) => {
            return connection
                .send(ServerMessage::JoinRejected {
                    code: join_rejection_code(&error).to_owned(),
                    message: "join rejected".to_owned(),
                })
                .await;
        }
    };

    registry.joined(id, room_id, session_id, participant.id);
    connection
        .send(ServerMessage::Joined {
            session_id: session_id.to_string(),
            participant_id: participant.id.to_string(),
        })
        .await?;
    state
        .speakers
        .replay_room(
            room_id,
            registry,
            state.stores.participants.as_ref(),
            state.stores.sessions.as_ref(),
        )
        .await;
    connection
        .send(ServerMessage::ActiveSpeakers {
            session_id: session_id.to_string(),
            participant_ids: state
                .speakers
                .current(session_id)
                .iter()
                .map(ToString::to_string)
                .collect(),
        })
        .await?;
    registry.broadcast_to_session_except(
        session_id,
        id,
        &ServerMessage::ParticipantJoined {
            session_id: session_id.to_string(),
            participant_id: participant.id.to_string(),
        },
    );
    drop(lifecycle_guard);
    Ok(())
}

/// Handle a `leave` frame for the connection's current participation.
pub(super) async fn handle_leave(
    connection: &mut Connection,
    registry: &ConnectionRegistry,
    id: Uuid,
    principal: &Principal,
    state: &WsState,
) -> Result<(), DisconnectReason> {
    let Some(handle) = registry.participation(id) else {
        return connection
            .send(ServerMessage::JoinRejected {
                code: "not_joined".to_owned(),
                message: "connection has not joined a session".to_owned(),
            })
            .await;
    };
    let session_id = handle.session_id;
    let participant_id = handle.participant_id;
    let lifecycle_lock = registry.lifecycle_lock(session_id, principal.actor_id);
    let lifecycle_guard = lifecycle_lock.lock().await;
    if !registry.left_if_current(id, handle) {
        drop(lifecycle_guard);
        return connection
            .send(ServerMessage::JoinRejected {
                code: "not_joined".to_owned(),
                message: "connection has not joined a session".to_owned(),
            })
            .await;
    }
    let remaining = registry.connection_count_for_participant(session_id, principal.actor_id);
    // Only the last local device ends durable participation. Serialize that
    // transition with joins so a reconnect cannot reuse a row being left.
    let outcome = if remaining == 0 {
        leave_session(
            state.stores.participants.as_ref(),
            state.stores.participation.as_ref(),
            LeaveSession {
                actor: principal.actor_id,
                participant_id,
            },
        )
        .await
    } else {
        Ok(())
    };
    if outcome.is_ok() && remaining == 0 {
        registry.broadcast_to_session_except(
            session_id,
            id,
            &ServerMessage::ParticipantLeft {
                session_id: session_id.to_string(),
                participant_id: participant_id.to_string(),
            },
        );
    }
    drop(lifecycle_guard);
    state.speakers.prune(registry);
    close_owned_transports(connection, state).await;
    match outcome {
        Ok(()) => {
            connection
                .send(ServerMessage::Left {
                    session_id: session_id.to_string(),
                    participant_id: participant_id.to_string(),
                })
                .await?;
            Ok(())
        }
        // A concurrent leave wins; the connection is already detached.
        Err(error) => {
            tracing::debug!(?error, participant_id = %participant_id, "leave rejected");
            connection
                .send(ServerMessage::JoinRejected {
                    code: "leave_rejected".to_owned(),
                    message: "leave rejected".to_owned(),
                })
                .await
        }
    }
}

/// Only this connection's transports are closed; a second device remains live.
pub(in crate::http::ws::connection) async fn close_owned_transports(
    connection: &mut Connection,
    state: &WsState,
) {
    for (session_id, room_id, transport_id) in connection.take_transports() {
        let media = match state.media_for(session_id, room_id).await {
            Ok(media) => media,
            Err(error) => {
                tracing::warn!(?error, %room_id, %transport_id, "cannot resolve transport media node");
                continue;
            }
        };
        let media_room = if state.placement.is_some() {
            crate::http::ws::placement::media_room_id(session_id)
        } else {
            room_id
        };
        if let Err(error) = media
            .close_transport(media_room, transport_id.clone())
            .await
        {
            tracing::warn!(?error, %room_id, %transport_id, "failed to close disconnected transport");
        }
    }
}

/// Map a join failure to a stable machine-readable code.
///
/// The message never carries internal detail: a non-member must not learn
/// whether the session exists.
fn join_rejection_code(error: &SessionOperationError) -> &'static str {
    match error {
        SessionOperationError::NotFound => "not_found",
        SessionOperationError::Forbidden => "forbidden",
        SessionOperationError::NotLive => "not_live",
        SessionOperationError::Conflict
        | SessionOperationError::Domain(_)
        | SessionOperationError::Participant(_) => "conflict",
        SessionOperationError::Store(_) | SessionOperationError::Participation(_) => "unavailable",
        SessionOperationError::Admission(_) => "admission_unavailable",
        SessionOperationError::Validation(_) => "invalid_request",
    }
}
