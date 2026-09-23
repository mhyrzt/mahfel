use super::error::SessionOperationError;
use crate::ports::{RoomStore, SessionStore};
use crate::rooms::require_manager;
use mahfel_domain::{ActorId, Session, SessionId};

#[derive(Debug, Clone)]
pub struct GetSession {
    pub actor: ActorId,
    pub session_id: SessionId,
}

/// Load one session for a room manager.
pub async fn get_session<R, S>(
    sessions: &R,
    rooms: &S,
    command: GetSession,
) -> Result<Session, SessionOperationError>
where
    R: SessionStore + ?Sized,
    S: RoomStore + ?Sized,
{
    let session = sessions
        .get(command.session_id)
        .await?
        .ok_or(SessionOperationError::NotFound)?;
    require_manager(rooms, session.room_id, command.actor).await?;
    Ok(session)
}
