//! Session participation frames (M4-02).
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

    // Deny by default: the application guard authorizes against membership and
    // visibility, and M4-06 admission policy extends this same path.
    let participant = match join_session(
        state.stores.participants.as_ref(),
        state.stores.participation.as_ref(),
        state.stores.admissions.as_ref(),
        state.stores.sessions.as_ref(),
        state.stores.rooms.as_ref(),
        state.stores.blocks.as_ref(),
        JoinSession {
            actor: principal.actor_id,
            session_id,
            origin: AdmissionOrigin::Join,
        },
    )
    .await
    {
        Ok(JoinOutcome::Joined(participant)) => participant,
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

    registry.joined(id, session_id, participant.id);
    connection
        .send(ServerMessage::Joined {
            session_id: session_id.to_string(),
            participant_id: participant.id.to_string(),
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
    let Some((session_id, participant_id)) = registry.participation(id) else {
        return connection
            .send(ServerMessage::JoinRejected {
                code: "not_joined".to_owned(),
                message: "connection has not joined a session".to_owned(),
            })
            .await;
    };

    // Whether this is the last connection the participant holds here. Detached
    // before the count, so this connection is not counted against itself.
    registry.left(id);
    let remaining = registry.connection_count_for_participant(session_id, principal.actor_id);

    // A participant is present while at least one connection is live (AC 7), so
    // a device closing while another is streaming must not end the
    // participation or tell the roster the participant left.
    if remaining > 0 {
        connection
            .send(ServerMessage::Left {
                session_id: session_id.to_string(),
                participant_id: participant_id.to_string(),
            })
            .await?;
        return Ok(());
    }

    let outcome = leave_session(
        state.stores.participants.as_ref(),
        state.stores.participation.as_ref(),
        LeaveSession {
            actor: principal.actor_id,
            participant_id,
        },
    )
    .await;
    match outcome {
        Ok(()) => {
            connection
                .send(ServerMessage::Left {
                    session_id: session_id.to_string(),
                    participant_id: participant_id.to_string(),
                })
                .await?;
            registry.broadcast_to_session_except(
                session_id,
                id,
                &ServerMessage::ParticipantLeft {
                    session_id: session_id.to_string(),
                    participant_id: participant_id.to_string(),
                },
            );
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

/// Map a join failure to a stable machine-readable code.
///
/// The message never carries internal detail: a non-member must not learn
/// whether the session exists (AC 2).
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
    }
}
