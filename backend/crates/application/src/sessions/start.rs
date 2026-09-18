use crate::ports::{SessionStore, StoreError};
use mahfel_domain::{ClassroomSession, ClassroomSessionId, SessionError};
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct StartSession {
    pub session_id: ClassroomSessionId,
}

#[derive(Debug, Error)]
pub enum StartSessionError {
    #[error("session not found")]
    NotFound,
    #[error("invalid session transition: {0}")]
    Domain(#[from] SessionError),
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

pub async fn execute<R>(
    sessions: &R,
    command: StartSession,
) -> Result<ClassroomSession, StartSessionError>
where
    R: SessionStore,
{
    let mut session = sessions
        .get(command.session_id)
        .await?
        .ok_or(StartSessionError::NotFound)?;
    session.start(OffsetDateTime::now_utc())?;
    sessions.save(&session).await?;
    Ok(session)
}
