use super::ManagerGuardError;
use crate::error::{ApplicationError, ErrorKind};
use crate::ports::{RoomStore, SessionStore, StoreError};
use crate::rooms::require_manager;
use mahfel_domain::{ActorId, RoomId, Session};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct ListSessions {
    pub actor: ActorId,
    pub room_id: RoomId,
}

#[derive(Debug, Error)]
pub enum ListSessionsError {
    #[error("room not found")]
    NotFound,
    #[error("permission denied")]
    Forbidden,
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

impl From<ManagerGuardError> for ListSessionsError {
    fn from(error: ManagerGuardError) -> Self {
        match error {
            ManagerGuardError::NotFound => Self::NotFound,
            ManagerGuardError::Forbidden => Self::Forbidden,
            ManagerGuardError::Store(error) => Self::Store(error),
        }
    }
}

impl ApplicationError for ListSessionsError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::NotFound => ErrorKind::NotFound,
            Self::Forbidden => ErrorKind::Forbidden,
            Self::Store(_) => ErrorKind::Internal,
        }
    }
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
) -> Result<Vec<Session>, ListSessionsError>
where
    R: SessionStore + ?Sized,
    S: RoomStore + ?Sized,
{
    require_manager(rooms, command.room_id, command.actor).await?;
    Ok(sessions.list_for_room(command.room_id).await?)
}
