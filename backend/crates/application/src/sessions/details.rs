use super::error::SessionOperationError;
use super::guard::require_manage_session;
use crate::ports::{Principal, RoomStore, SessionDetailsFields, SessionStore};
use mahfel_domain::{Session, SessionDurationMinutes, SessionId, SessionTitle};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct UpdateSessionDetails {
    pub principal: Principal,
    pub session_id: SessionId,
    /// Outer `None` leaves the field unchanged; inner `None` clears it.
    pub title: Option<Option<String>>,
    pub scheduled_start_at: Option<Option<OffsetDateTime>>,
    pub scheduled_duration_minutes: Option<Option<u32>>,
}

/// Change advisory details in any session state, including ended/cancelled.
pub async fn update_session_details<S, R>(
    sessions: &S,
    rooms: &R,
    command: UpdateSessionDetails,
) -> Result<Session, SessionOperationError>
where
    S: SessionStore + ?Sized,
    R: RoomStore + ?Sized,
{
    let mut session = sessions
        .get(command.session_id)
        .await?
        .ok_or(SessionOperationError::NotFound)?;
    require_manage_session(rooms, session.room_id, &command.principal).await?;

    let fields = SessionDetailsFields {
        title: command.title.is_some(),
        scheduled_start_at: command.scheduled_start_at.is_some(),
        scheduled_duration_minutes: command.scheduled_duration_minutes.is_some(),
    };

    let title = command
        .title
        .map(|value| value.map(SessionTitle::new).transpose())
        .transpose()?
        .unwrap_or(session.title.clone());
    let duration = command
        .scheduled_duration_minutes
        .map(|value| value.map(SessionDurationMinutes::new).transpose())
        .transpose()?
        .unwrap_or(session.scheduled_duration_minutes);
    let start = command
        .scheduled_start_at
        .unwrap_or(session.scheduled_start_at);
    session.set_details(title, start, duration, OffsetDateTime::now_utc());
    let session = sessions.save_details(&session, fields).await?;
    tracing::info!(
        target: "audit",
        action = "session.details.update",
        actor_id = %command.principal.actor_id,
        room_id = %session.room_id,
        session_id = %session.id,
        outcome = "ok"
    );
    Ok(session)
}
