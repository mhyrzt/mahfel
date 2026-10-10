//! Approve or reject a waiting principal.
//!
//! What approval grants depends on what the request asked for, and the two
//! cases are not interchangeable:
//!
//! - a **session** request asks to attend one live session, so approval creates
//!   the participant and opens the participation interval;
//! - a **room** request asks to join the room itself, so approval grants durable
//!   membership. It requires no live session, because the room outlives any one
//!   session and that is the point of a room link (ADR 0021).

use super::error::AdmissionOperationError;
use super::request::target_kind;
use crate::ports::{AdmissionStore, RoomStore, SessionStore};
use mahfel_domain::{
    ActorId, AdmissionRequest, AdmissionRequestStatus, AdmissionTarget, Participant, RoomId,
    SessionStatus,
};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct AdmissionDecision {
    /// The principal deciding. Must manage the target's room.
    pub actor: ActorId,
    pub target: AdmissionTarget,
    /// The principal whose request is being decided.
    pub subject: ActorId,
}

/// Authorize a session approval without mutating the request or participant.
/// Placement runs after this check and before the approval's durable writes.
pub async fn preflight_session_admit<A, T, R>(
    admissions: &A,
    sessions: &T,
    rooms: &R,
    command: &AdmissionDecision,
) -> Result<RoomId, AdmissionOperationError>
where
    A: AdmissionStore + ?Sized,
    T: SessionStore + ?Sized,
    R: RoomStore + ?Sized,
{
    let AdmissionTarget::Session(session_id) = command.target else {
        return Err(AdmissionOperationError::NotFound);
    };
    let request = load_decidable(admissions, command.target, command.subject).await?;
    let room_id = authorize(rooms, sessions, request.target, command.actor).await?;
    let session = sessions
        .get(session_id)
        .await?
        .ok_or(AdmissionOperationError::NotFound)?;
    if session.status != SessionStatus::Live {
        return Err(AdmissionOperationError::NotLive);
    }
    Ok(room_id)
}

/// Approve a waiting principal.
///
/// The store commits the decision and grant atomically. A failed grant leaves
/// the request pending; a rejection that wins the race grants nothing.
///
/// # Errors
/// [`AdmissionOperationError::NotFound`] when the target is missing,
/// [`AdmissionOperationError::Forbidden`] when the caller does not manage its
/// room, [`AdmissionOperationError::NotWaiting`] when nothing is pending,
/// [`AdmissionOperationError::AlreadyDecided`] on a racing decision.
pub async fn admit<A, T, R>(
    admissions: &A,
    sessions: &T,
    rooms: &R,
    command: AdmissionDecision,
    now: OffsetDateTime,
) -> Result<AdmissionGrant, AdmissionOperationError>
where
    A: AdmissionStore + ?Sized,
    T: SessionStore + ?Sized,
    R: RoomStore + ?Sized,
{
    let request = load_decidable(admissions, command.target, command.subject).await?;
    let room_id = authorize(rooms, sessions, request.target, command.actor).await?;

    if let AdmissionTarget::Session(session_id) = request.target {
        let session = sessions
            .get(session_id)
            .await?
            .ok_or(AdmissionOperationError::NotFound)?;
        if session.status != SessionStatus::Live {
            return Err(AdmissionOperationError::NotLive);
        }
    }
    let granted = admissions
        .approve_pending(request.id, command.actor, now)
        .await?
        .ok_or(AdmissionOperationError::AlreadyDecided)?;

    tracing::info!(
        target: "audit",
        action = "admission.admit",
        actor_id = %command.actor,
        room_id = %room_id,
        session_id = ?request.target.session_id(),
        target_kind = target_kind(request.target),
        target_actor_id = %command.subject,
        request_id = %request.id,
        origin = request.origin.as_str(),
        outcome = "ok"
    );

    Ok(granted)
}

/// Reject a waiting principal. Terminates the waiting state and grants nothing.
///
/// # Errors
/// [`AdmissionOperationError::NotFound`] when the target is missing,
/// [`AdmissionOperationError::Forbidden`] when the caller does not manage its
/// room, [`AdmissionOperationError::NotWaiting`] when nothing is pending.
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
    let request = load_decidable(admissions, command.target, command.subject).await?;
    // A rejection needs no live session for either target: refusing to grant
    // access is always meaningful.
    let room_id = authorize(rooms, sessions, request.target, command.actor).await?;

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
        room_id = %room_id,
        session_id = ?request.target.session_id(),
        target_kind = target_kind(request.target),
        target_actor_id = %command.subject,
        request_id = %request.id,
        origin = request.origin.as_str(),
        outcome = "ok"
    );

    Ok(())
}

/// What approving a request granted.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AdmissionGrant {
    /// Attendance in one session, carrying no room membership.
    Participant(Participant),
    /// Durable membership in the room.
    Membership { room_id: mahfel_domain::RoomId },
}

impl AdmissionGrant {
    #[must_use]
    pub fn participant(&self) -> Option<&Participant> {
        match self {
            Self::Participant(participant) => Some(participant),
            Self::Membership { .. } => None,
        }
    }
}

async fn load_decidable<A>(
    admissions: &A,
    target: AdmissionTarget,
    subject: ActorId,
) -> Result<AdmissionRequest, AdmissionOperationError>
where
    A: AdmissionStore + ?Sized,
{
    admissions
        .get_pending(target, subject)
        .await?
        .ok_or(AdmissionOperationError::NotWaiting)
}

/// Resolve the room that authorizes the decision, or deny.
///
/// A session target resolves through its session so the manager check uses the
/// room the session belongs to.
async fn authorize<T, R>(
    rooms: &R,
    sessions: &T,
    target: AdmissionTarget,
    actor: ActorId,
) -> Result<mahfel_domain::RoomId, AdmissionOperationError>
where
    T: SessionStore + ?Sized,
    R: RoomStore + ?Sized,
{
    let room_id = match target {
        AdmissionTarget::Session(session_id) => {
            sessions
                .get(session_id)
                .await?
                .ok_or(AdmissionOperationError::NotFound)?
                .room_id
        }
        AdmissionTarget::Room(room_id) => room_id,
    };
    crate::rooms::require_manager(rooms, room_id, actor).await?;
    Ok(room_id)
}
