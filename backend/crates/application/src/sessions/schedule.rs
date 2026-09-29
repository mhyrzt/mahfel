use super::error::SessionOperationError;
use crate::ports::Principal;
use crate::ports::{RoomStore, SessionStore, StoreError};
use crate::sessions::guard::require_manage_session;
use mahfel_domain::{RoomId, Session, SessionDurationMinutes, SessionTitle};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct ScheduleSession {
    pub principal: Principal,
    pub room_id: RoomId,
    pub auto_record: bool,
    pub title: Option<String>,
    pub scheduled_start_at: Option<OffsetDateTime>,
    pub scheduled_duration_minutes: Option<u32>,
}

/// Manager-only: schedule a new session for a room.
///
/// The database enforces at most one live session per room; a conflicting
/// insert surfaces as [`SessionOperationError::Conflict`].
pub async fn schedule_session<R, S>(
    sessions: &R,
    rooms: &S,
    command: ScheduleSession,
) -> Result<Session, SessionOperationError>
where
    R: SessionStore + ?Sized,
    S: RoomStore + ?Sized,
{
    require_manage_session(rooms, command.room_id, &command.principal).await?;
    let now = OffsetDateTime::now_utc();
    let mut session = Session::schedule(command.room_id, command.scheduled_start_at, now);
    session.set_auto_record(command.auto_record)?;
    session.set_details(
        command.title.map(SessionTitle::new).transpose()?,
        command.scheduled_start_at,
        command
            .scheduled_duration_minutes
            .map(SessionDurationMinutes::new)
            .transpose()?,
        now,
    );
    sessions
        .insert(&session)
        .await
        .map_err(|error| match error {
            StoreError::Conflict(_) => SessionOperationError::Conflict,
            other => SessionOperationError::Store(other),
        })?;
    tracing::info!(
        target: "audit",
        action = "session.schedule",
        actor_id = %command.principal.actor_id,
        room_id = %session.room_id,
        session_id = %session.id,
        outcome = "ok"
    );
    Ok(session)
}
