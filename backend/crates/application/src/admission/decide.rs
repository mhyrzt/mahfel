//! Approve or reject a waiting principal.
//!
//! Approval is the moment access is granted: it creates the participant and
//! opens the participation interval, so the durable record starts at the
//! decision rather than at the earlier request. Rejection terminates the
//! waiting state and grants nothing.

use super::error::AdmissionOperationError;
use crate::ports::{AdmissionStore, ParticipantStore, ParticipationStore, RoomStore, SessionStore};
use mahfel_domain::{
    ActorId, AdmissionRequestStatus, Participant, ParticipationInterval, SessionId, SessionStatus,
};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct AdmissionDecision {
    /// The principal deciding. Must hold `admit_participant`.
    pub actor: ActorId,
    pub session_id: SessionId,
    /// The principal whose request is being decided.
    pub target: ActorId,
}

/// Approve a waiting principal and admit them to the session.
///
/// The participant row is created before the request is marked admitted, so a
/// failure leaves the request pending and retryable rather than reporting a
/// decision that did not grant access.
///
/// # Errors
/// [`AdmissionOperationError::NotFound`] when the session or request is missing,
/// [`AdmissionOperationError::Forbidden`] when the caller lacks the capability,
/// [`AdmissionOperationError::NotWaiting`] when the principal has no pending
/// request, [`AdmissionOperationError::AlreadyDecided`] on a racing decision.
pub async fn admit<P, Q, A, T, R>(
    participants: &P,
    participation: &Q,
    admissions: &A,
    sessions: &T,
    rooms: &R,
    command: AdmissionDecision,
    now: OffsetDateTime,
) -> Result<Participant, AdmissionOperationError>
where
    P: ParticipantStore + ?Sized,
    Q: ParticipationStore + ?Sized,
    A: AdmissionStore + ?Sized,
    T: SessionStore + ?Sized,
    R: RoomStore + ?Sized,
{
    let session = sessions
        .get(command.session_id)
        .await?
        .ok_or(AdmissionOperationError::NotFound)?;
    crate::rooms::require_manager(rooms, session.room_id, command.actor).await?;
    if session.status != SessionStatus::Live {
        return Err(AdmissionOperationError::Domain(
            mahfel_domain::SessionError::CannotEnd(session.status.as_str()),
        ));
    }

    let request = admissions
        .get_pending(command.session_id, command.target)
        .await?
        .ok_or(AdmissionOperationError::NotWaiting)?;

    // Reuse an existing active participant rather than duplicating it, so a
    // principal who reconnected while waiting keeps one identity.
    let participant = match participants
        .get_active(command.session_id, command.target)
        .await?
    {
        Some(existing) => existing,
        None => {
            let participant = Participant::join(command.session_id, command.target, now);
            match participants.insert(&participant).await {
                Ok(()) => participant,
                Err(crate::ports::StoreError::Conflict(_)) => participants
                    .get_active(command.session_id, command.target)
                    .await?
                    .ok_or(AdmissionOperationError::NotFound)?,
                Err(error) => return Err(error.into()),
            }
        }
    };

    let interval =
        ParticipationInterval::open(participant.id, command.session_id, command.target, now);
    match participation.open_interval(&interval).await {
        Ok(()) => {}
        // An interval is already open for this participant; it still records
        // the connection, so this is not a failure.
        Err(crate::ports::StoreError::Conflict(_)) => {}
        Err(error) => return Err(error.into()),
    }

    let decided = admissions
        .decide_if_pending(
            request.id,
            AdmissionRequestStatus::Admitted,
            Some(command.actor),
            now,
        )
        .await?;
    if !decided {
        return Err(AdmissionOperationError::AlreadyDecided);
    }

    tracing::info!(
        target: "audit",
        action = "admission.admit",
        actor_id = %command.actor,
        session_id = %command.session_id,
        room_id = %session.room_id,
        target_actor_id = %command.target,
        request_id = %request.id,
        participant_id = %participant.id,
        outcome = "ok"
    );

    Ok(participant)
}

/// Reject a waiting principal. Terminates the waiting state and grants nothing.
///
/// # Errors
/// [`AdmissionOperationError::NotFound`] when the session is missing,
/// [`AdmissionOperationError::Forbidden`] when the caller lacks the capability,
/// [`AdmissionOperationError::NotWaiting`] when nothing is pending.
pub async fn reject<A, T, R>(
    admissions: &A,
    sessions: &T,
    rooms: &R,
    command: AdmissionDecision,
    now: OffsetDateTime,
) -> Result<(), AdmissionOperationError>
where
    A: AdmissionStore + ?Sized,
    T: SessionStore + ?Sized,
    R: RoomStore + ?Sized,
{
    let session = sessions
        .get(command.session_id)
        .await?
        .ok_or(AdmissionOperationError::NotFound)?;
    crate::rooms::require_manager(rooms, session.room_id, command.actor).await?;

    let request = admissions
        .get_pending(command.session_id, command.target)
        .await?
        .ok_or(AdmissionOperationError::NotWaiting)?;

    let decided = admissions
        .decide_if_pending(
            request.id,
            AdmissionRequestStatus::Rejected,
            Some(command.actor),
            now,
        )
        .await?;
    if !decided {
        return Err(AdmissionOperationError::AlreadyDecided);
    }

    tracing::info!(
        target: "audit",
        action = "admission.reject",
        actor_id = %command.actor,
        session_id = %command.session_id,
        room_id = %session.room_id,
        target_actor_id = %command.target,
        request_id = %request.id,
        outcome = "ok"
    );

    Ok(())
}
