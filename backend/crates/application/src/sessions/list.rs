use super::error::SessionOperationError;
use crate::ports::{RoomStore, SessionStore};
use crate::rooms::require_manager;
use mahfel_domain::{ActorId, RoomId, Session};

#[derive(Debug, Clone)]
pub struct ListSessions {
    pub actor: ActorId,
    pub room_id: RoomId,
}

/// List a room's sessions, newest first.
///
/// Scoped to room managers in this task. Member visibility is deferred to the
/// capability model (M4-04), which will express the read capability once it
/// exists rather than hardcoding a role here.
pub async fn list_sessions<R, S>(
    sessions: &R,
    rooms: &S,
    command: ListSessions,
) -> Result<Vec<Session>, SessionOperationError>
where
    R: SessionStore + ?Sized,
    S: RoomStore + ?Sized,
{
    require_manager(rooms, command.room_id, command.actor).await?;
    Ok(sessions.list_for_room(command.room_id).await?)
}
