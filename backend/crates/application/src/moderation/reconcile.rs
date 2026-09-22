//! Reconcile a principal's producers against their current capabilities.
//!
//! One operation rather than a sweep per trigger. It re-resolves the
//! principal's capabilities for a room and closes the producers whose source is
//! no longer permitted:
//!
//! - a membership removal withdraws every media capability, so every source is
//!   closed;
//! - a block bars entry, so every source is closed;
//! - a demotion withdraws moderation but not participation, so nothing is
//!   closed, because a member keeps `PublishMedia` and `ShareScreen`.
//!
//! That last case is why this is a reconcile rather than a "close everything"
//! sweep: the correct action is a function of what the principal still holds, so
//! computing it from the capability set stays correct if a role is ever
//! introduced that lacks a media capability.
//!
//! Room-scoped rather than session-scoped, because a room has at most one live
//! session at a time and the media plane keys producers by room. Reconciling a
//! room therefore reaches exactly the live session, with no iteration and no
//! risk of closing a producer that belongs to an older one.
//!
//! Closing rather than pausing is deliberate. A pause would leave a producer
//! that nobody resumes: the client cannot be relied on to keep a paused producer
//! alive across an arbitrary interval, unlike a mute, where the target is
//! connected and expected to unmute soon.

use super::error::ModerationError;
use crate::ports::{Capability, MediaGateway, MembershipStore};
use mahfel_domain::{ActorId, ProducerSource, RoomId};

/// What a reconcile closed.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct ReconcileOutcome {
    pub closed_microphone: bool,
    pub closed_camera: bool,
    pub closed_screen: bool,
}

impl ReconcileOutcome {
    #[must_use]
    pub const fn closed_anything(self) -> bool {
        self.closed_microphone || self.closed_camera || self.closed_screen
    }
}

/// The capability that permits publishing one source.
///
/// A screen share is a separate capability from speaking, which is why a source
/// is needed to pick between them.
#[must_use]
pub const fn capability_for(source: ProducerSource) -> Capability {
    match source {
        ProducerSource::Microphone | ProducerSource::Camera => Capability::PublishMedia,
        ProducerSource::Screen => Capability::ShareScreen,
    }
}

/// Every source a principal may no longer publish in this room.
///
/// Returns nothing when the capabilities still cover every source, which is the
/// demotion case: the caller then makes no media call at all.
#[must_use]
fn forbidden_sources(granted: &crate::ports::CapabilitySet) -> Vec<ProducerSource> {
    [
        ProducerSource::Microphone,
        ProducerSource::Camera,
        ProducerSource::Screen,
    ]
    .into_iter()
    .filter(|source| !granted.has(capability_for(*source)))
    .collect()
}

/// Close any producer whose source the principal may no longer publish.
///
/// # Errors
/// [`ModerationError::Forbidden`] when the membership cannot be read, which
/// resolves to deny rather than to a grant, and
/// [`ModerationError::Enforcement`] when a close fails.
pub async fn reconcile_media<S, M>(
    memberships: &S,
    media: &M,
    actor: ActorId,
    room_id: RoomId,
) -> Result<ReconcileOutcome, ModerationError>
where
    S: MembershipStore + ?Sized,
    M: MediaGateway + ?Sized,
{
    let granted =
        crate::policy::resolve_room_capabilities(memberships, &principal_for(actor), room_id)
            .await
            .map_err(|_| ModerationError::Forbidden)?;

    let forbidden = forbidden_sources(&granted);
    if forbidden.is_empty() {
        // Nothing was withdrawn, so there is nothing to close and no media call
        // to make. A demotion lands here.
        return Ok(ReconcileOutcome::default());
    }

    let mut outcome = ReconcileOutcome::default();
    for source in forbidden {
        let closed = media
            .close_participant_source(room_id, actor, source)
            .await?;
        if closed == 0 {
            continue;
        }
        match source {
            ProducerSource::Microphone => outcome.closed_microphone = true,
            ProducerSource::Camera => outcome.closed_camera = true,
            ProducerSource::Screen => outcome.closed_screen = true,
        }
    }

    if outcome.closed_anything() {
        tracing::info!(
            target: "audit",
            action = "moderation.reconcile",
            actor_id = %actor,
            room_id = %room_id,
            closed_microphone = outcome.closed_microphone,
            closed_camera = outcome.closed_camera,
            closed_screen = outcome.closed_screen,
            outcome = "ok"
        );
    }

    Ok(outcome)
}

/// A principal carrying only the actor, for a membership lookup.
///
/// The resolver reads membership rather than global roles for a room-scoped
/// capability, so an empty role set is the honest description: this reconcile
/// decides from the room membership alone.
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
