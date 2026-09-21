use super::error::SessionOperationError;
use crate::ports::{RoomStore, SessionStore, StoreError};
use crate::rooms::require_manager;
use mahfel_domain::{ActorId, Session, SessionId};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct StartSession {
    pub actor: ActorId,
    pub session_id: SessionId,
}

/// Manager-only: move a scheduled session to live.
pub async fn start_session<R, S>(
    sessions: &R,
    rooms: &S,
    command: StartSession,
) -> Result<Session, SessionOperationError>
where
    R: SessionStore + ?Sized,
    S: RoomStore + ?Sized,
{
    let mut session = sessions
        .get(command.session_id)
        .await?
        .ok_or(SessionOperationError::NotFound)?;
    require_manager(rooms, session.room_id, command.actor).await?;
    session.start(OffsetDateTime::now_utc())?;
    sessions.save(&session).await.map_err(|error| match error {
        StoreError::Conflict(_) => SessionOperationError::Conflict,
        other => SessionOperationError::Store(other),
    })?;
    tracing::info!(
        target: "audit",
        action = "session.start",
        actor_id = %command.actor,
        room_id = %session.room_id,
        session_id = %session.id,
        outcome = "ok"
    );
    Ok(session)
}
