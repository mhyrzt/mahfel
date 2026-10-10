//! Two mutes, and they are not degrees of the same thing:
//!
//! - a **temporary** mute is a reset the target clears themselves;
//! - a **permanent** mute is held until someone holding `MuteOthers` lifts it.
//!
//! Neither survives a rejoin, because leaving is terminal and a new join is a
//! new participation. Keeping someone out after they leave is a block, not a
//! stronger mute.
//!
//! Enforcement reaches the media plane: the target's producers for the muted
//! source are paused, not closed, so an unmute resumes them without the target
//! renegotiating transport.

use super::error::ModerationError;
use crate::ports::{Capability, MediaGateway, MembershipStore, ParticipantStore, SessionStore};
use mahfel_domain::{
    ActorId, MuteKind, Participant, ParticipantStatus, ProducerSource, RoomId, SessionId,
};
use time::OffsetDateTime;

/// Which source a mute applies to.
///
/// Muting audio and stopping a screen share are separate operations. The
/// default is the one a plain "mute" means.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MutedSource {
    /// Captured audio, the plain meaning of mute.
    Microphone,
    Camera,
    Screen,
}

impl MutedSource {
    #[must_use]
    pub const fn as_source(self) -> ProducerSource {
        match self {
            Self::Microphone => ProducerSource::Microphone,
            Self::Camera => ProducerSource::Camera,
            Self::Screen => ProducerSource::Screen,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MuteCommand {
    /// The principal acting. May be the target clearing their own mute.
    pub actor: crate::ports::Principal,
    pub session_id: SessionId,
    /// Whose audio is affected.
    pub subject: ActorId,
    pub kind: MuteKind,
    pub source: MutedSource,
}

/// What a mute changed.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct MuteOutcome {
    pub kind: MuteKind,
    pub source: ProducerSource,
    /// How many producers the enforcement paused or resumed. Zero is normal: a
    /// target who has published nothing still carries the mute, so it applies
    /// when they next publish.
    pub producers_affected: usize,
    /// The durable mute was recorded but the media plane could not be reached,
    /// so the target's producer may still be running. Surfaced rather than
    /// hidden, because it is the difference between a mute that is enforced and
    /// one that is only recorded.
    pub degraded: bool,
}

/// Mute a participant, or upgrade an existing mute to a firmer one.
///
/// # Errors
/// [`ModerationError::NotFound`] when the session is missing or the subject has
/// no active participant, [`ModerationError::Forbidden`] when the caller lacks
/// `MuteOthers` for the session's room.
pub async fn mute<P, T, S, M>(
    participants: &P,
    sessions: &T,
    memberships: &S,
    media: &M,
    command: MuteCommand,
    now: OffsetDateTime,
) -> Result<MuteOutcome, ModerationError>
where
    P: ParticipantStore + ?Sized,
    T: SessionStore + ?Sized,
    S: MembershipStore + ?Sized,
    M: MediaGateway + ?Sized,
{
    if command.actor.actor_id == command.subject {
        return Err(ModerationError::Forbidden);
    }
    let session = sessions
        .get(command.session_id)
        .await?
        .ok_or(ModerationError::NotFound)?;
    let room_id = session.room_id;

    // Capability first, so an unauthorized caller cannot use the difference
    // between one failure and another to learn who is in the session.
    crate::policy::require_room_capability(
        memberships,
        &command.actor,
        room_id,
        Capability::MuteOthers,
    )
    .await?;

    let mut participant = require_active_participant(
        participants,
        command.session_id,
        command.subject,
        ModerationError::NotFound,
    )
    .await?;

    participant.mute(command.kind, command.actor.actor_id, now)?;
    participants.save(&participant).await?;

    // Enforcement is best-effort for the outcome: the durable mute is already
    // recorded, and a media outage must not withhold the frame that tells the
    // target to stop sending. Reporting a failure instead would leave the
    // participant muted in the database while their client kept publishing and
    // nobody was told — the worst of both. The failure is audited as degraded so
    // an operator can reconcile.
    let (producers_affected, degraded) = match enforce(
        media,
        room_id,
        command.subject,
        command.source.as_source(),
        Enforcement::Pause,
    )
    .await
    {
        Ok(affected) => (affected, false),
        Err(error) => {
            tracing::error!(
                target: "audit",
                action = "moderation.mute.enforcement_failed",
                actor_id = %command.actor.actor_id,
                room_id = %room_id,
                session_id = %command.session_id,
                target_actor_id = %command.subject,
                participant_id = %participant.id,
                error = %error,
                outcome = "degraded"
            );
            (0, true)
        }
    };

    tracing::info!(
        target: "audit",
        action = "moderation.mute",
        actor_id = %command.actor.actor_id,
        room_id = %room_id,
        session_id = %command.session_id,
        target_actor_id = %command.subject,
        participant_id = %participant.id,
        mute_kind = command.kind.as_str(),
        source = command.source.as_source().as_str(),
        producers_affected,
        degraded,
        outcome = "ok"
    );

    Ok(MuteOutcome {
        kind: command.kind,
        source: command.source.as_source(),
        producers_affected,
        degraded,
    })
}

/// Clear a mute.
///
/// A temporary mute is the target's own to clear. A permanent one requires
/// `MuteOthers`, and the refusal is enforced by the aggregate so it cannot be
/// skipped by a caller that forgets to check.
///
/// # Errors
/// [`ModerationError::NotFound`] when the session or participant is missing,
/// [`ModerationError::NotMuted`] when nothing is muted,
/// [`ModerationError::PermanentMuteNeedsModerator`] on a self-unmute of a
/// permanent mute.
pub async fn unmute<P, T, S, M>(
    participants: &P,
    sessions: &T,
    memberships: &S,
    media: &M,
    command: MuteCommand,
    now: OffsetDateTime,
) -> Result<MuteOutcome, ModerationError>
where
    P: ParticipantStore + ?Sized,
    T: SessionStore + ?Sized,
    S: MembershipStore + ?Sized,
    M: MediaGateway + ?Sized,
{
    let session = sessions
        .get(command.session_id)
        .await?
        .ok_or(ModerationError::NotFound)?;
    let room_id = session.room_id;

    // A self-unmute is allowed without any capability; clearing someone else's
    // mute requires the same grant that imposed it.
    let is_self = command.actor.actor_id == command.subject;
    let may_moderate = if is_self {
        // Still resolved, because a moderator clearing someone else's permanent
        // mute is a different case; here it only decides whether a permanent
        // self-unmute would be permitted, which the aggregate then refuses.
        false
    } else {
        crate::policy::require_room_capability(
            memberships,
            &command.actor,
            room_id,
            Capability::MuteOthers,
        )
        .await?;
        true
    };

    let mut participant = require_active_participant(
        participants,
        command.session_id,
        command.subject,
        ModerationError::NotFound,
    )
    .await?;

    let kind = participant.mute_kind.ok_or(ModerationError::NotMuted)?;
    participant.unmute(may_moderate, now)?;
    participants.save(&participant).await?;

    let (producers_affected, degraded) = match enforce(
        media,
        room_id,
        command.subject,
        command.source.as_source(),
        Enforcement::Resume,
    )
    .await
    {
        Ok(affected) => (affected, false),
        Err(error) => {
            tracing::error!(
                target: "audit",
                action = "moderation.unmute.enforcement_failed",
                actor_id = %command.actor.actor_id,
                room_id = %room_id,
                session_id = %command.session_id,
                target_actor_id = %command.subject,
                error = %error,
                outcome = "degraded"
            );
            (0, true)
        }
    };

    tracing::info!(
        target: "audit",
        action = "moderation.unmute",
        actor_id = %command.actor.actor_id,
        room_id = %room_id,
        session_id = %command.session_id,
        target_actor_id = %command.subject,
        participant_id = %participant.id,
        cleared_kind = kind.as_str(),
        source = command.source.as_source().as_str(),
        producers_affected,
        degraded,
        outcome = "ok"
    );

    Ok(MuteOutcome {
        kind,
        source: command.source.as_source(),
        producers_affected,
        degraded,
    })
}

/// The subject's active participant, or a deliberate failure.
///
/// This is the authoritative identity check. It runs before any media call,
/// which is what lets the media plane stay free of a participant registry.
pub(crate) async fn require_active_participant<P>(
    participants: &P,
    session_id: SessionId,
    subject: ActorId,
    missing: ModerationError,
) -> Result<Participant, ModerationError>
where
    P: ParticipantStore + ?Sized,
{
    let participant = participants
        .get_active(session_id, subject)
        .await?
        .filter(|participant| participant.status == ParticipantStatus::Active)
        .ok_or(missing)?;
    Ok(participant)
}

enum Enforcement {
    Pause,
    Resume,
}

/// Apply a mute to the media plane for one source.
///
/// Only the named source is touched, so muting a microphone leaves a camera and
/// a screen share alone. A media failure surfaces rather than being swallowed:
/// the durable mute is already recorded, so an operator can reconcile, but the
/// caller must not be told enforcement succeeded when it did not.
async fn enforce<M>(
    media: &M,
    room_id: RoomId,
    subject: ActorId,
    source: ProducerSource,
    enforcement: Enforcement,
) -> Result<usize, ModerationError>
where
    M: MediaGateway + ?Sized,
{
    let affected = match enforcement {
        Enforcement::Pause => {
            media
                .pause_participant_source(room_id, subject, source)
                .await
        }
        Enforcement::Resume => {
            media
                .resume_participant_source(room_id, subject, source)
                .await
        }
    };
    // Provenance is preserved rather than flattened: the caller sees that the
    // durable state changed and enforcement did not, which is what an operator
    // needs to reconcile.
    affected.map_err(ModerationError::from)
}
