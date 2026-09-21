use super::error::SessionOperationError;
use crate::ports::{RoomStore, SessionStore, StoreError};
use crate::rooms::require_manager;
use mahfel_domain::{ActorId, RoomId, Session};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct ScheduleSession {
    pub actor: ActorId,
    pub room_id: RoomId,
    pub scheduled_start_at: Option<OffsetDateTime>,
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
            StoreError::Conflict(_) => SessionOperationError::Conflict,
            other => SessionOperationError::Store(other),
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
