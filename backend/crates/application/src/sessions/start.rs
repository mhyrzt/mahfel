use super::error::SessionOperationError;
use crate::ports::Principal;
use crate::ports::{RoomStore, SessionStartClaim, SessionStore, StoreError};
use crate::sessions::guard::require_manage_session;
use mahfel_domain::{Session, SessionId};
use std::{future::Future, time::Duration};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct StartSession {
    pub principal: Principal,
    pub session_id: SessionId,
}

/// Move a scheduled session to live when the caller may manage it.
pub async fn start_session<R, S>(
    sessions: &R,
    rooms: &S,
    command: StartSession,
) -> Result<Session, SessionOperationError>
where
    R: SessionStore + ?Sized,
    S: RoomStore + ?Sized,
{
    let mut session = sessions
        .get(command.session_id)
        .await?
        .ok_or(SessionOperationError::NotFound)?;
    require_manage_session(rooms, session.room_id, &command.principal).await?;
    session.start(OffsetDateTime::now_utc())?;
    sessions.save(&session).await.map_err(|error| match error {
        StoreError::Conflict(_) => SessionOperationError::Conflict,
        other => SessionOperationError::Store(other),
    })?;
    tracing::info!(
        target: "audit",
        action = "session.start",
        actor_id = %command.principal.actor_id,
        room_id = %session.room_id,
        session_id = %session.id,
        outcome = "ok"
    );
    Ok(session)
}

/// Start a session under a durable capture readiness lease. The readiness
/// callback must check the selected media node and private recording storage.
pub async fn start_session_with_recording<R, S, F, Fut>(
    sessions: &R,
    rooms: &S,
    command: StartSession,
    readiness: F,
) -> Result<Session, SessionOperationError>
where
    R: SessionStore + ?Sized,
    S: RoomStore + ?Sized,
    F: FnOnce(SessionId, mahfel_domain::RoomId) -> Fut,
    Fut: Future<Output = bool>,
{
    let session = sessions
        .get(command.session_id)
        .await?
        .ok_or(SessionOperationError::NotFound)?;
    require_manage_session(rooms, session.room_id, &command.principal).await?;
    let result = sessions
        .claim_start(session.id)
        .await
        .map_err(|error| match error {
            StoreError::Conflict(_) => SessionOperationError::Conflict,
            other => SessionOperationError::Store(other),
        })?;
    let session = match result {
        SessionStartClaim::Started(session) => session,
        SessionStartClaim::RecordingLease(token) => {
            let healthy = tokio::time::timeout(
                Duration::from_secs(8),
                readiness(session.id, session.room_id),
            )
            .await
            .unwrap_or(false);
            if !healthy {
                sessions.release_recorded_start(session.id, token).await?;
                return Err(SessionOperationError::RecordingUnavailable);
            }
            sessions
                .commit_recorded_start(session.id, token)
                .await
                .map_err(|error| match error {
                    StoreError::Conflict(_) => SessionOperationError::Conflict,
                    other => SessionOperationError::Store(other),
                })?
        }
    };
    tracing::info!(target: "audit", action = "session.start", actor_id = %command.principal.actor_id, room_id = %session.room_id, session_id = %session.id, outcome = "ok");
    Ok(session)
}
