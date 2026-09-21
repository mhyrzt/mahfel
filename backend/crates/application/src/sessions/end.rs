use super::error::SessionOperationError;
use crate::ports::{ParticipationStore, RoomStore, SessionStore, StoreError};
use crate::rooms::require_manager;
use mahfel_domain::{ActorId, Session, SessionId};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct EndSession {
    pub actor: ActorId,
    pub session_id: SessionId,
}

/// Manager-only: move a live session to ended, then close open intervals.
///
/// The sweep stops every open interval's clock at the session end, so no
/// interval outlives its session (AC 5). It runs after the session is durably
/// ended, and is idempotent, so a retried end does not move an end time.
pub async fn end_session<R, S, Q>(
    sessions: &R,
    rooms: &S,
    participation: &Q,
    command: EndSession,
) -> Result<Session, SessionOperationError>
where
    R: SessionStore + ?Sized,
    S: RoomStore + ?Sized,
    Q: ParticipationStore + ?Sized,
{
    let now = OffsetDateTime::now_utc();
    let mut session = sessions
        .get(command.session_id)
        .await?
        .ok_or(SessionOperationError::NotFound)?;
    require_manager(rooms, session.room_id, command.actor).await?;
    session.end(now)?;
    sessions.save(&session).await.map_err(|error| match error {
        StoreError::Conflict(_) => SessionOperationError::Conflict,
        other => SessionOperationError::Store(other),
    })?;

    let closed = crate::participation::close_session_intervals(participation, session.id, now)
        .await
        .map_err(|error| match error {
            crate::participation::ParticipationError::Store(error) => {
                SessionOperationError::Store(error)
            }
            other => {
                // A failure to sweep is not a failure to end the session; the
                // session is already ended durably. Surface it for the caller
                // without leaving the interval mid-flight.
                tracing::error!(?other, session_id = %session.id, "participation sweep failed");
                SessionOperationError::Store(StoreError::Unexpected(other.to_string()))
            }
        })?;

    tracing::info!(
        target: "audit",
        action = "session.end",
        actor_id = %command.actor,
        room_id = %session.room_id,
        session_id = %session.id,
        closed_intervals = closed,
        outcome = "ok"
    );
    Ok(session)
}
