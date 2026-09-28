//! Remove and block are orthogonal, and collapsing them would erase the
//! distinction:
//!
//! - **remove** ends this participation now and the principal may rejoin;
//! - **block** is a durable bar, so a blocked principal cannot rejoin while it
//!   is active.
//!
//! Both resolve the target across all of its connections: producers are closed
//! once for the principal rather than per connection, and the participant is
//! ended once.

use super::error::ModerationError;
use crate::ports::{
    BlockStore, Capability, MediaGateway, MembershipStore, ParticipantStore, RoomStore,
    SessionStore,
};
use mahfel_domain::{ActorId, Block, BlockScope, SessionId};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct RemoveCommand {
    pub actor: ActorId,
    pub session_id: SessionId,
    /// Whose participation ends.
    pub subject: ActorId,
}

/// End a participant's participation in one session.
///
/// The participant is loaded first, so a subject who does not exist or is no
/// longer active fails with [`ModerationError::NotFound`] before any media call.
/// That is the authoritative identity check: the media plane holds no
/// participant registry, so this is the only layer that can answer it.
///
/// # Errors
/// [`ModerationError::NotFound`] when the session is missing or the subject has
/// no active participant, [`ModerationError::Forbidden`] when the caller lacks
/// `RemoveParticipant`.
pub async fn remove_participant<P, T, S, M>(
    participants: &P,
    sessions: &T,
    memberships: &S,
    media: &M,
    command: RemoveCommand,
    now: OffsetDateTime,
) -> Result<(), ModerationError>
where
    P: ParticipantStore + ?Sized,
    T: SessionStore + ?Sized,
    S: MembershipStore + ?Sized,
    M: MediaGateway + ?Sized,
{
    if command.actor == command.subject {
        return Err(ModerationError::Forbidden);
    }
    let session = sessions
        .get(command.session_id)
        .await?
        .ok_or(ModerationError::NotFound)?;
    let room_id = session.room_id;

    crate::policy::require_actor_capability(
        memberships,
        command.actor,
        room_id,
        Capability::RemoveParticipant,
    )
    .await?;

    let mut participant = super::mute::require_active_participant(
        participants,
        command.session_id,
        command.subject,
        ModerationError::NotFound,
    )
    .await?;

    // Terminal, so a later join is a new participation rather than a revival.
    participant.remove(now)?;
    participants.save(&participant).await?;

    // Media cleanup last, and best-effort for the same reason as a mute: the
    // participation is already ended, so failing the call would misreport a
    // removal that happened, while a silent failure would leave a removed
    // principal publishing. It is audited as degraded instead.
    if let Err(error) = media.close_participant(room_id, command.subject).await {
        tracing::error!(
            target: "audit",
            action = "moderation.remove.enforcement_failed",
            actor_id = %command.actor,
            room_id = %room_id,
            session_id = %command.session_id,
            target_actor_id = %command.subject,
            participant_id = %participant.id,
            error = %error,
            outcome = "degraded"
        );
    }

    tracing::info!(
        target: "audit",
        action = "moderation.remove",
        actor_id = %command.actor,
        room_id = %room_id,
        session_id = %command.session_id,
        target_actor_id = %command.subject,
        participant_id = %participant.id,
        outcome = "ok"
    );

    Ok(())
}

#[derive(Debug, Clone)]
pub struct BlockCommand {
    pub actor: ActorId,
    pub scope: BlockScope,
    pub subject: ActorId,
    pub reason: Option<String>,
    /// `None` means until explicitly lifted; a timestamp lets it lapse on its
    /// own, which is what a one-off meeting wants.
    pub expires_at: Option<OffsetDateTime>,
}

/// What a block operation produced.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct BlockOutcome {
    /// Whether a new block was imposed. `false` means an existing active block
    /// was extended instead.
    pub created: bool,
    /// How many sessions the subject was ejected from, because a block that
    /// left them publishing would not be a block. Zero when they were not in
    /// the session.
    pub ejected_from: usize,
}

/// Bar a principal from a room or a session.
///
/// Re-blocking an already-blocked principal **extends** the existing block
/// rather than inserting a second row, so there is never more than one active
/// block to resolve and a re-block has a well-defined meaning.
///
/// A block also composes removal: the subject is ejected from every session the
/// block covers, so their participation ends and their producers close. A block
/// bars rejoining, and one that left the subject publishing would not be a
/// block.
///
/// # Errors
/// [`ModerationError::NotFound`] when the target does not exist,
/// [`ModerationError::Forbidden`] when the caller lacks `RemoveParticipant`
/// (the capability that authorizes barring someone from the session's room),
/// [`ModerationError::Block`] for a malformed expiry.
#[allow(clippy::too_many_arguments)]
pub async fn block<R, T, P, B, S, M>(
    rooms: &R,
    sessions: &T,
    participants: &P,
    blocks: &B,
    memberships: &S,
    media: &M,
    command: BlockCommand,
    now: OffsetDateTime,
) -> Result<BlockOutcome, ModerationError>
where
    R: RoomStore + ?Sized,
    T: SessionStore + ?Sized,
    P: ParticipantStore + ?Sized,
    B: BlockStore + ?Sized,
    S: MembershipStore + ?Sized,
    M: MediaGateway + ?Sized,
{
    if command.actor == command.subject {
        return Err(ModerationError::Forbidden);
    }
    let room_id = super::scope::authorize_scope(rooms, sessions, &command.scope).await?;
    crate::policy::require_actor_capability(
        memberships,
        command.actor,
        room_id,
        Capability::RemoveParticipant,
    )
    .await?;

    let created = if let Some(existing) = blocks.get_active(command.scope, command.subject).await? {
        blocks.extend(existing.id, command.expires_at).await?;
        tracing::info!(
            target: "audit",
            action = "moderation.block.extend",
            actor_id = %command.actor,
            room_id = %room_id,
            session_id = ?command.scope.session_id(),
            target_actor_id = %command.subject,
            block_id = %existing.id,
            outcome = "ok"
        );
        false
    } else {
        let block = Block::new(
            command.scope,
            command.subject,
            command.actor,
            command.reason,
            command.expires_at,
            now,
        )?;
        blocks.insert(&block).await?;
        tracing::info!(
            target: "audit",
            action = "moderation.block",
            actor_id = %command.actor,
            room_id = %room_id,
            session_id = ?command.scope.session_id(),
            target_actor_id = %command.subject,
            block_id = %block.id,
            expires = ?command.expires_at,
            outcome = "ok"
        );
        true
    };

    // Eject from the sessions the block covers. A session block covers its one
    // session; a room block covers every session in the room, which is at most
    // one live session because a room enforces that.
    let covered = match command.scope {
        BlockScope::Session(session_id) => sessions
            .get(session_id)
            .await?
            .map(|session| vec![session])
            .unwrap_or_default(),
        BlockScope::Room(room_id) => sessions.list_for_room(room_id).await?,
    };

    let mut ejected_from = 0;
    for session in covered {
        let Some(mut participant) = participants.get_active(session.id, command.subject).await?
        else {
            continue;
        };
        // Terminal, like a removal: a later join is a new participation, and the
        // block is what refuses it.
        participant.remove(now)?;
        participants.save(&participant).await?;
        media.close_participant(room_id, command.subject).await?;
        ejected_from += 1;
    }

    if ejected_from > 0 {
        tracing::info!(
            target: "audit",
            action = "moderation.block.eject",
            actor_id = %command.actor,
            room_id = %room_id,
            target_actor_id = %command.subject,
            ejected_from,
            outcome = "ok"
        );
    }

    Ok(BlockOutcome {
        created,
        ejected_from,
    })
}

/// Lift a block before it lapses.
///
/// # Errors
/// [`ModerationError::NotFound`] when the scope's room is missing or invisible,
/// [`ModerationError::AlreadyLifted`] when there is no active block to lift.
pub async fn unblock<R, T, B, S>(
    rooms: &R,
    sessions: &T,
    blocks: &B,
    memberships: &S,
    command: BlockCommand,
    now: OffsetDateTime,
) -> Result<(), ModerationError>
where
    R: RoomStore + ?Sized,
    T: SessionStore + ?Sized,
    B: BlockStore + ?Sized,
    S: MembershipStore + ?Sized,
{
    if command.actor == command.subject {
        return Err(ModerationError::Forbidden);
    }
    let room_id = super::scope::authorize_scope(rooms, sessions, &command.scope).await?;
    crate::policy::require_actor_capability(
        memberships,
        command.actor,
        room_id,
        Capability::RemoveParticipant,
    )
    .await?;

    let active = blocks
        .get_active(command.scope, command.subject)
        .await?
        .ok_or(ModerationError::AlreadyLifted)?;
    let lifted = blocks.lift(active.id, command.actor, now).await?;
    if !lifted {
        return Err(ModerationError::AlreadyLifted);
    }

    tracing::info!(
        target: "audit",
        action = "moderation.unblock",
        actor_id = %command.actor,
        room_id = %room_id,
        session_id = ?command.scope.session_id(),
        target_actor_id = %command.subject,
        block_id = %active.id,
        outcome = "ok"
    );

    Ok(())
}
