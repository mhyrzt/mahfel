use super::ManagerGuardError;
use crate::error::{ApplicationError, ErrorKind};
use crate::ports::{RoomStore, SessionStore, StoreError};
use crate::rooms::require_manager;
use mahfel_domain::{ActorId, Session, SessionError, SessionId};
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct EndSession {
    pub actor: ActorId,
    pub session_id: SessionId,
}

#[derive(Debug, Error)]
pub enum EndSessionError {
    #[error("session not found")]
    NotFound,
    #[error("permission denied")]
    Forbidden,
    #[error("a live session already exists for this room")]
    Conflict,
    #[error("invalid session transition: {0}")]
    Domain(#[from] SessionError),
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

impl From<ManagerGuardError> for EndSessionError {
    fn from(error: ManagerGuardError) -> Self {
        match error {
            ManagerGuardError::NotFound => Self::NotFound,
            ManagerGuardError::Forbidden => Self::Forbidden,
            ManagerGuardError::Store(error) => Self::Store(error),
        }
    }
}

impl ApplicationError for EndSessionError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::NotFound => ErrorKind::NotFound,
            Self::Forbidden => ErrorKind::Forbidden,
            Self::Conflict => ErrorKind::Conflict,
            Self::Domain(_) => ErrorKind::Conflict,
            Self::Store(_) => ErrorKind::Internal,
        }
    }
}

/// Manager-only: move a live session to ended.
pub async fn end_session<R, S>(
    sessions: &R,
    rooms: &S,
    command: EndSession,
) -> Result<Session, EndSessionError>
where
    R: SessionStore + ?Sized,
    S: RoomStore + ?Sized,
{
    let mut session = sessions
        .get(command.session_id)
        .await?
        .ok_or(EndSessionError::NotFound)?;
    require_manager(rooms, session.room_id, command.actor).await?;
    session.end(OffsetDateTime::now_utc())?;
    sessions.save(&session).await.map_err(|error| match error {
        StoreError::Conflict(_) => EndSessionError::Conflict,
        other => EndSessionError::Store(other),
    })?;
    tracing::info!(
        target: "audit",
        action = "session.end",
        actor_id = %command.actor,
        room_id = %session.room_id,
        session_id = %session.id,
        outcome = "ok"
    );
    Ok(session)
}
