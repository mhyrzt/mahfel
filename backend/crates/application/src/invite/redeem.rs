//! A code is a bearer capability, so redemption is where the security posture
//! actually has to hold:
//!
//! - attempts are counted before the code is looked at, so a limited attempt
//!   costs the same as an invalid one;
//! - an unknown, expired, revoked, or dangling code is one identical failure,
//!   so the endpoint cannot be used to enumerate valid codes;
//! - the raw code never reaches a log line.
//!
//! Redemption does not join. It resolves the target and, unless the caller
//! already has access, queues a request. Joining stays the join path's job
//! (ADR 0021).

use super::error::InviteOperationError;
use super::rate_limit::AttemptLimiter;
use crate::admission::error::AdmissionOperationError;
use crate::admission::request::{AdmissionOutcome, EnqueueAdmission, enqueue_admission};
use crate::ports::{AdmissionStore, CoordinationStore, InviteStore, RoomStore, SessionStore};
use mahfel_domain::{
    ActorId, AdmissionOrigin, AdmissionRequestId, AdmissionTarget, InviteTarget, RoomStatus,
    SessionId,
};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct RedeemInviteLink {
    pub actor: ActorId,
    /// The code as the user typed or pasted it. Normalized before lookup.
    pub code: String,
    /// The caller's source address, for the per-address attempt budget. `None`
    /// when the deployment cannot attribute one, which leaves the per-principal
    /// budget, the stricter of the two, in force.
    pub source_address: Option<String>,
}

/// What redeeming a code resolved to.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Redeemed {
    /// The caller already has access, so nothing needs approving. A member who
    /// redeems a session link for a session that is not live yet still gets
    /// this: the code is fine, and joining is a later concern.
    AlreadyMember { target: InviteTarget },
    /// A decision is required before the caller may participate.
    Waiting {
        target: InviteTarget,
        request_id: AdmissionRequestId,
    },
}

impl Redeemed {
    #[must_use]
    pub const fn target(self) -> InviteTarget {
        match self {
            Self::AlreadyMember { target } | Self::Waiting { target, .. } => target,
        }
    }

    #[must_use]
    pub const fn is_waiting(self) -> bool {
        matches!(self, Self::Waiting { .. })
    }
}

/// Resolve a code and queue the caller if a decision is required.
///
/// # Errors
/// [`InviteOperationError::NotFound`] for every unusable code, deliberately
/// indistinguishable from an unknown one. Lobby capacity surfaces as the
/// admission error, because that is a different fact about a code that is
/// genuinely valid.
#[allow(clippy::too_many_arguments)]
pub async fn redeem_invite_link<I, A, T, R, C>(
    invites: &I,
    admissions: &A,
    sessions: &T,
    rooms: &R,
    coordination: &C,
    limiter: &AttemptLimiter,
    command: RedeemInviteLink,
    now: OffsetDateTime,
) -> Result<Redeemed, RedeemError>
where
    I: InviteStore + ?Sized,
    A: AdmissionStore + ?Sized,
    T: SessionStore + ?Sized,
    R: RoomStore + ?Sized,
    C: CoordinationStore + ?Sized,
{
    // Counted before the code is looked at, so a limited attempt is
    // indistinguishable from an invalid one and cannot be probed for free.
    let decision = limiter
        .check(
            coordination,
            &principal_for(command.actor),
            command.source_address.as_deref(),
        )
        .await
        .map_err(|_| InviteOperationError::not_redeemable())?;
    if !decision.is_allowed() {
        return Err(InviteOperationError::not_redeemable().into());
    }

    let target = resolve_usable_target(invites, sessions, rooms, &command.code, now).await?;

    let room_id = match target {
        AdmissionTarget::Session(session_id) => {
            sessions
                .get(session_id)
                .await?
                .ok_or_else(InviteOperationError::not_redeemable)?
                .room_id
        }
        AdmissionTarget::Room(room_id) => room_id,
    };

    if rooms.get_member(room_id, command.actor).await?.is_some() {
        return Ok(Redeemed::AlreadyMember {
            target: invite_target_of(target),
        });
    }

    let outcome = enqueue_admission(
        admissions,
        sessions,
        rooms,
        EnqueueAdmission {
            actor: command.actor,
            target,
            origin: AdmissionOrigin::InviteLink,
        },
        now,
    )
    .await?;

    let AdmissionOutcome::Waiting(request) = outcome;
    tracing::info!(
        target: "audit",
        action = "invite.redeem",
        actor_id = %command.actor,
        room_id = %room_id,
        session_id = ?target.session_id(),
        request_id = %request.id,
        outcome = "waiting"
    );

    Ok(Redeemed::Waiting {
        target: invite_target_of(target),
        request_id: request.id,
    })
}

/// Resolve a code into a target, or fail with the one non-enumerating error.
///
/// Every rejection uses the same value: a malformed code, an unknown code, an
/// expired or revoked link, a target whose row is gone, and an archived room.
/// The code's validity and the target's existence are therefore both
/// unlearnable from the response.
async fn resolve_usable_target<I, T, R>(
    invites: &I,
    sessions: &T,
    rooms: &R,
    raw_code: &str,
    now: OffsetDateTime,
) -> Result<AdmissionTarget, InviteOperationError>
where
    I: InviteStore + ?Sized,
    T: SessionStore + ?Sized,
    R: RoomStore + ?Sized,
{
    let unusable = InviteOperationError::not_redeemable;
    let code = mahfel_domain::InviteCode::parse(raw_code).map_err(|_| unusable())?;
    let link = invites
        .get_by_code(code)
        .await?
        .filter(|link| link.is_redeemable_at(now))
        .ok_or_else(unusable)?;

    match link.target {
        InviteTarget::Session(session_id) => {
            // A link can outlive its session; a missing session must fail closed.
            sessions.get(session_id).await?.ok_or_else(unusable)?;
            Ok(AdmissionTarget::Session(session_id))
        }
        InviteTarget::Room(room_id) => {
            let room = rooms.get(room_id).await?.ok_or_else(unusable)?;
            // An archived room is closed to new members, so its links are dead.
            if room.status != RoomStatus::Active {
                return Err(unusable());
            }
            Ok(AdmissionTarget::Room(room_id))
        }
    }
}

/// Why redemption failed.
#[derive(Debug, thiserror::Error)]
pub enum RedeemError {
    /// Codes: every unusable one collapses to a single indistinguishable value.
    #[error("invite link not found")]
    NotRedeemable(#[from] InviteOperationError),
    #[error("admission failure: {0}")]
    Admission(#[from] AdmissionOperationError),
    #[error("storage failure: {0}")]
    Store(#[from] crate::ports::StoreError),
}

impl crate::error::ApplicationError for RedeemError {
    fn kind(&self) -> crate::error::ErrorKind {
        match self {
            Self::NotRedeemable(error) => error.kind(),
            Self::Admission(error) => error.kind(),
            Self::Store(_) => crate::error::ErrorKind::Internal,
        }
    }
}

/// The limiter keys on a principal, so it needs the same shape the resolver
/// uses. Built here rather than widening the limiter's signature to a bare id.
fn principal_for(actor: ActorId) -> crate::ports::Principal {
    crate::ports::Principal {
        actor_id: actor,
        kind: mahfel_domain::ActorKind::User,
        display_name: None,
        avatar_url: None,
        email: None,
        email_verified: false,
        username: None,
        roles: Vec::new(),
    }
}

/// The link-facing view of an admission target.
const fn invite_target_of(target: AdmissionTarget) -> InviteTarget {
    match target {
        AdmissionTarget::Session(session_id) => InviteTarget::Session(session_id),
        AdmissionTarget::Room(room_id) => InviteTarget::Room(room_id),
    }
}

/// The session a redeemed target is scoped to, for the response.
#[must_use]
pub const fn redeemed_session(target: InviteTarget) -> Option<SessionId> {
    match target {
        InviteTarget::Session(session_id) => Some(session_id),
        InviteTarget::Room(_) => None,
    }
}
