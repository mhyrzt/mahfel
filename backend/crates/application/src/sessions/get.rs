use super::ManagerGuardError;
use crate::error::{ApplicationError, ErrorKind};
use crate::ports::{RoomStore, SessionStore, StoreError};
use crate::rooms::require_manager;
use mahfel_domain::{ActorId, Session, SessionId};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct GetSession {
    pub actor: ActorId,
    pub session_id: SessionId,
}

#[derive(Debug, Error)]
pub enum GetSessionError {
    #[error("session not found")]
    NotFound,
    #[error("permission denied")]
    Forbidden,
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

impl From<ManagerGuardError> for GetSessionError {
    fn from(error: ManagerGuardError) -> Self {
        match error {
            ManagerGuardError::NotFound => Self::NotFound,
            ManagerGuardError::Forbidden => Self::Forbidden,
            ManagerGuardError::Store(error) => Self::Store(error),
        }
    }
}

impl ApplicationError for GetSessionError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::NotFound => ErrorKind::NotFound,
            Self::Forbidden => ErrorKind::Forbidden,
            Self::Store(_) => ErrorKind::Internal,
        }
    }
}

/// Load one session.
///
/// Scoped to room managers in this task, matching [`list_sessions`](super::list_sessions);
/// member visibility arrives with the capability model (M4-04).
pub async fn get_session<R, S>(
    sessions: &R,
    rooms: &S,
    command: GetSession,
) -> Result<Session, GetSessionError>
where
    R: SessionStore,
    S: RoomStore,
{
    let session = sessions
        .get(command.session_id)
        .await?
        .ok_or(GetSessionError::NotFound)?;
    require_manager(rooms, session.room_id, command.actor).await?;
    Ok(session)
}
