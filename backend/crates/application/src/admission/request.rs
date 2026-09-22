//! Queue a join for approval when a session requires it.
//!
//! This is what the join path calls when the effective policy is `lobby`: the
//! caller has already passed authorization, so the only remaining question is
//! whether someone approves. No participant is created here.

use super::error::AdmissionOperationError;
use crate::ports::{AdmissionStore, SessionStore, StoreError};
use mahfel_domain::{
    ActorId, AdmissionOrigin, AdmissionRequest, MAX_PENDING_PER_SESSION, SessionId,
};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct EnqueueAdmission {
    pub actor: ActorId,
    pub session_id: SessionId,
    pub origin: AdmissionOrigin,
}

/// What a join attempt resolved to.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AdmissionOutcome {
    /// A decision is required before this principal may participate.
    Waiting(AdmissionRequest),
}

/// Place a principal in a session's lobby.
///
/// Idempotent per principal: a caller who already has a pending request gets
/// that request back rather than a second queue entry, so a reconnect while
/// waiting does not consume another slot or lose its place.
///
/// # Errors
/// [`AdmissionOperationError::LobbyFull`] when the waiting set is at capacity,
/// which bounds a lobby that any authorized caller can otherwise fill.
pub async fn enqueue_admission<A, T>(
    admissions: &A,
    sessions: &T,
    command: EnqueueAdmission,
    now: OffsetDateTime,
) -> Result<AdmissionOutcome, AdmissionOperationError>
where
    A: AdmissionStore + ?Sized,
    T: SessionStore + ?Sized,
{
    if let Some(existing) = admissions
        .get_pending(command.session_id, command.actor)
        .await?
    {
        return Ok(AdmissionOutcome::Waiting(existing));
    }

    let pending = admissions.count_pending(command.session_id).await?;
    if pending >= MAX_PENDING_PER_SESSION as u64 {
        return Err(AdmissionOperationError::LobbyFull);
    }

    let request = AdmissionRequest::new(command.session_id, command.actor, command.origin, now);
    match admissions.insert(&request).await {
        Ok(()) => {}
        // A racing join queued first; reuse its request rather than failing.
        Err(StoreError::Conflict(_)) => {
            let existing = admissions
                .get_pending(command.session_id, command.actor)
                .await?
                .ok_or(AdmissionOperationError::NotFound)?;
            return Ok(AdmissionOutcome::Waiting(existing));
        }
        Err(other) => return Err(other.into()),
    }

    let session = sessions
        .get(command.session_id)
        .await?
        .ok_or(AdmissionOperationError::NotFound)?;

    tracing::info!(
        target: "audit",
        action = "admission.request",
        actor_id = %command.actor,
        session_id = %command.session_id,
        room_id = %session.room_id,
        request_id = %request.id,
        origin = command.origin.as_str(),
        outcome = "ok"
    );

    Ok(AdmissionOutcome::Waiting(request))
}

/// The principal's outstanding request, if it is still waiting.
///
/// # Errors
/// [`AdmissionOperationError::NotFound`] when nothing is pending.
pub async fn wait_for_admission<A>(
    admissions: &A,
    session_id: SessionId,
    actor: ActorId,
) -> Result<AdmissionRequest, AdmissionOperationError>
where
    A: AdmissionStore + ?Sized,
{
    admissions
        .get_pending(session_id, actor)
        .await?
        .ok_or(AdmissionOperationError::NotFound)
}
