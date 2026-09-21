use super::ManagerGuardError;
use crate::error::{ApplicationError, ErrorKind};
use crate::ports::{RoomStore, SessionStore, StoreError};
use crate::rooms::require_manager;
use mahfel_domain::{ActorId, RoomId, Session};
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct ScheduleSession {
    pub actor: ActorId,
    pub room_id: RoomId,
    pub scheduled_start_at: Option<OffsetDateTime>,
}

#[derive(Debug, Error)]
pub enum ScheduleSessionError {
    #[error("room not found")]
    NotFound,
    #[error("permission denied")]
    Forbidden,
    #[error("a live session already exists for this room")]
    Conflict,
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

impl From<ManagerGuardError> for ScheduleSessionError {
    fn from(error: ManagerGuardError) -> Self {
        match error {
            ManagerGuardError::NotFound => Self::NotFound,
            ManagerGuardError::Forbidden => Self::Forbidden,
            ManagerGuardError::Store(error) => Self::Store(error),
        }
    }
}

impl ApplicationError for ScheduleSessionError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::NotFound => ErrorKind::NotFound,
            Self::Forbidden => ErrorKind::Forbidden,
            Self::Conflict => ErrorKind::Conflict,
            Self::Store(_) => ErrorKind::Internal,
        }
    }
}

/// Manager-only: schedule a new session for a room.
///
/// The database enforces at most one live session per room; a conflicting
/// insert surfaces as [`ScheduleSessionError::Conflict`].
pub async fn schedule_session<R, S>(
    sessions: &R,
    rooms: &S,
    command: ScheduleSession,
) -> Result<Session, ScheduleSessionError>
where
    R: SessionStore + ?Sized,
    S: RoomStore + ?Sized,
{
    require_manager(rooms, command.room_id, command.actor).await?;
    let session = Session::schedule(
        command.room_id,
        command.scheduled_start_at,
        OffsetDateTime::now_utc(),
    );
    sessions
        .insert(&session)
        .await
        .map_err(|error| match error {
            StoreError::Conflict(_) => ScheduleSessionError::Conflict,
            other => ScheduleSessionError::Store(other),
        })?;
    tracing::info!(
        target: "audit",
        action = "session.schedule",
        actor_id = %command.actor,
        room_id = %session.room_id,
        session_id = %session.id,
        outcome = "ok"
    );
    Ok(session)
}
