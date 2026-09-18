use crate::ports::{SessionStore, StoreError};
use mahfel_domain::{ClassroomSession, RoomId};
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct ScheduleSession {
    pub room_id: RoomId,
    pub scheduled_start_at: Option<OffsetDateTime>,
}

#[derive(Debug, Error)]
pub enum ScheduleSessionError {
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

pub async fn execute<R>(
    sessions: &R,
    command: ScheduleSession,
) -> Result<ClassroomSession, ScheduleSessionError>
where
    R: SessionStore,
{
    let session = ClassroomSession::schedule(
        command.room_id,
        command.scheduled_start_at,
        OffsetDateTime::now_utc(),
    );
    sessions.insert(&session).await?;
    Ok(session)
}
