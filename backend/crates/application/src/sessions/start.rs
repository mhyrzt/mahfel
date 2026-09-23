use super::error::SessionOperationError;
use crate::ports::Principal;
use crate::ports::{RoomStore, SessionStore, StoreError};
use crate::sessions::guard::require_manage_session;
use mahfel_domain::{Session, SessionId};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct StartSession {
    pub principal: Principal,
    pub session_id: SessionId,
}

/// Move a scheduled session to live when the caller may manage it.
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
    require_manage_session(rooms, session.room_id, &command.principal).await?;
    session.start(OffsetDateTime::now_utc())?;
    sessions.save(&session).await.map_err(|error| match error {
        StoreError::Conflict(_) => SessionOperationError::Conflict,
        other => SessionOperationError::Store(other),
    })?;
    tracing::info!(
        target: "audit",
        action = "session.start",
        actor_id = %command.principal.actor_id,
        room_id = %session.room_id,
        session_id = %session.id,
        outcome = "ok"
    );
    Ok(session)
}
