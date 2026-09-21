use super::error::SessionOperationError;
use crate::ports::{ParticipantStore, RoomStore, SessionStore, StoreError};
use mahfel_domain::{ActorId, Participant, ParticipantError, SessionId, SessionStatus};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct JoinSession {
    pub actor: ActorId,
    pub session_id: SessionId,
}

/// Join a live session, reusing an existing participant on reconnect.
///
/// The single join path: M4-06 layers admission policy on top of the same
/// guard, so it must not be bypassed by another entry point.
pub async fn join_session<P, R, S>(
    participants: &P,
    sessions: &R,
    rooms: &S,
    command: JoinSession,
) -> Result<Participant, SessionOperationError>
where
    P: ParticipantStore + ?Sized,
    R: SessionStore + ?Sized,
    S: RoomStore + ?Sized,
{
    let session = sessions
        .get(command.session_id)
        .await?
        .ok_or(SessionOperationError::NotFound)?;
    if session.status != SessionStatus::Live {
        return Err(SessionOperationError::NotLive);
    }
    super::require_joinable(rooms, session.room_id, command.actor).await?;

    if let Some(existing) = participants
        .get_active(command.session_id, command.actor)
        .await?
    {
        tracing::info!(
            target: "audit",
            action = "participant.reconnect",
            actor_id = %command.actor,
            session_id = %command.session_id,
            participant_id = %existing.id,
            outcome = "ok"
        );
        return Ok(existing);
    }

    let participant =
        Participant::join(command.session_id, command.actor, OffsetDateTime::now_utc());
    participants
        .insert(&participant)
        .await
        .map_err(|error| match error {
            StoreError::Conflict(_) => {
                SessionOperationError::Participant(ParticipantError::AlreadyActive)
            }
            other => SessionOperationError::Store(other),
        })?;
    tracing::info!(
        target: "audit",
        action = "participant.join",
        actor_id = %command.actor,
        session_id = %command.session_id,
        participant_id = %participant.id,
        outcome = "ok"
    );
    Ok(participant)
}
