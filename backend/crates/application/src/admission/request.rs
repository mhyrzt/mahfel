//! Queue a join for approval.
//!
//! This is what the join path calls when the effective policy is `lobby`, and
//! what invite-link redemption calls for either target. The caller has already
//! passed authorization, so the only remaining question is whether someone
//! approves. No participant is created here.

use super::error::AdmissionOperationError;
use crate::ports::{AdmissionStore, RoomStore, SessionStore, StoreError};
use mahfel_domain::{
    ActorId, AdmissionOrigin, AdmissionRequest, AdmissionTarget, MAX_PENDING_PER_SESSION,
};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct EnqueueAdmission {
    pub actor: ActorId,
    pub target: AdmissionTarget,
    pub origin: AdmissionOrigin,
}

/// What a join attempt resolved to.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AdmissionOutcome {
    /// A decision is required before this principal may participate.
    Waiting(AdmissionRequest),
}

/// Place a principal in a lobby.
///
/// Idempotent per principal and target: a caller who already has a pending
/// request gets that request back rather than a second queue entry, so a
/// reconnect while waiting does not consume another slot or lose its place.
///
/// # Errors
/// [`AdmissionOperationError::LobbyFull`] when the waiting set is at capacity,
/// which bounds a lobby that any authorized caller can otherwise fill.
pub async fn enqueue_admission<A, T, R>(
    admissions: &A,
    sessions: &T,
    rooms: &R,
    command: EnqueueAdmission,
    now: OffsetDateTime,
) -> Result<AdmissionOutcome, AdmissionOperationError>
where
    A: AdmissionStore + ?Sized,
    T: SessionStore + ?Sized,
    R: RoomStore + ?Sized,
{
    if let Some(existing) = admissions
        .get_pending(command.target, command.actor)
        .await?
    {
        return Ok(AdmissionOutcome::Waiting(existing));
    }

    let pending = admissions.count_pending(command.target).await?;
    if pending >= MAX_PENDING_PER_SESSION as u64 {
        return Err(AdmissionOperationError::LobbyFull);
    }

    let request = AdmissionRequest::new(command.target, command.actor, command.origin, now);
    match admissions.insert(&request).await {
        Ok(()) => {}
        // A racing join queued first; reuse its request rather than failing.
        Err(StoreError::Conflict(_)) => {
            let existing = admissions
                .get_pending(command.target, command.actor)
                .await?
                .ok_or(AdmissionOperationError::NotFound)?;
            return Ok(AdmissionOutcome::Waiting(existing));
        }
        Err(other) => return Err(other.into()),
    }

    let room_id = match command.target {
        AdmissionTarget::Session(session_id) => {
            sessions
                .get(session_id)
                .await?
                .ok_or(AdmissionOperationError::NotFound)?
                .room_id
        }
        AdmissionTarget::Room(room_id) => {
            // The room must exist, so a request never points at nothing.
            rooms
                .get(room_id)
                .await?
                .ok_or(AdmissionOperationError::NotFound)?;
            room_id
        }
    };

    tracing::info!(
        target: "audit",
        action = "admission.request",
        actor_id = %command.actor,
        room_id = %room_id,
        session_id = ?command.target.session_id(),
        request_id = %request.id,
        origin = command.origin.as_str(),
        target_kind = target_kind(command.target),
        outcome = "ok"
    );

    Ok(AdmissionOutcome::Waiting(request))
}

/// The principal's outstanding request for a target, if it is still waiting.
///
/// # Errors
/// [`AdmissionOperationError::NotFound`] when nothing is pending.
pub async fn wait_for_admission<A>(
    admissions: &A,
    target: AdmissionTarget,
    actor: ActorId,
) -> Result<AdmissionRequest, AdmissionOperationError>
where
    A: AdmissionStore + ?Sized,
{
    admissions
        .get_pending(target, actor)
        .await?
        .ok_or(AdmissionOperationError::NotFound)
}

pub(crate) fn target_kind(target: AdmissionTarget) -> &'static str {
    match target {
        AdmissionTarget::Session(_) => "session",
        AdmissionTarget::Room(_) => "room",
    }
}
