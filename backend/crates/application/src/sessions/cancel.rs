use crate::ports::{SessionStore, StoreError};
use mahfel_domain::{ClassroomSession, ClassroomSessionId, SessionError};
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct CancelSession {
    pub session_id: ClassroomSessionId,
}

#[derive(Debug, Error)]
pub enum CancelSessionError {
    #[error("session not found")]
    NotFound,
    #[error("invalid session transition: {0}")]
    Domain(#[from] SessionError),
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

pub async fn execute<R>(
    sessions: &R,
    command: CancelSession,
) -> Result<ClassroomSession, CancelSessionError>
where
    R: SessionStore,
{
    let mut session = sessions
        .get(command.session_id)
        .await?
        .ok_or(CancelSessionError::NotFound)?;
    session.cancel(OffsetDateTime::now_utc())?;
    sessions.save(&session).await?;
    Ok(session)
}
