use super::error::SessionOperationError;
use crate::ports::{RoomStore, SessionStore};
use mahfel_domain::{ActorId, RoomId, Session};

#[derive(Debug, Clone)]
pub struct ListSessions {
    pub actor: ActorId,
    pub room_id: RoomId,
    pub status: Option<mahfel_domain::SessionStatus>,
    pub limit: u64,
    pub offset: u64,
}

/// List a room's sessions, newest first, for a room member.
pub async fn list_sessions<R, S>(
    sessions: &R,
    rooms: &S,
    command: ListSessions,
) -> Result<Vec<Session>, SessionOperationError>
where
    R: SessionStore + ?Sized,
    S: RoomStore + ?Sized,
{
    if rooms
        .get_member(command.room_id, command.actor)
        .await?
        .is_none()
    {
        return Err(SessionOperationError::NotFound);
    }
    Ok(sessions
        .list_for_room_page(
            command.room_id,
            command.status,
            command.limit.min(100),
            command.offset,
        )
        .await?)
}
