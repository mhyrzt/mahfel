//! Issue an invite link for a room or a session.
//!
//! Management authority for a session is reached through its room, which is the
//! engine's existing model: a session is managed by whoever manages its room.

use super::code::insert_with_unique_code;
use super::error::InviteOperationError;
use crate::ports::{InviteStore, RoomStore, SessionStore};
use crate::rooms::require_manager;
use mahfel_domain::{
    ActorId, DEFAULT_EXPIRY_DAYS, InviteCode, InviteLink, InviteTarget, RoomId, SessionId,
};
use time::OffsetDateTime;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InviteTargetSelector {
    Room(RoomId),
    Session(SessionId),
}

#[derive(Debug, Clone)]
pub struct CreateInviteLink {
    pub actor: ActorId,
    pub target: InviteTargetSelector,
    /// `None` uses [`DEFAULT_EXPIRY_DAYS`].
    pub expiry_days: Option<i64>,
}

/// # Errors
/// [`InviteOperationError::NotFound`] when the target is missing or invisible,
/// [`InviteOperationError::Forbidden`] when the caller does not manage it.
pub async fn create_invite_link<S, T, R>(
    invites: &S,
    sessions: &T,
    rooms: &R,
    command: CreateInviteLink,
    now: OffsetDateTime,
) -> Result<InviteLink, InviteOperationError>
where
    S: InviteStore + ?Sized,
    T: SessionStore + ?Sized,
    R: RoomStore + ?Sized,
{
    let (target, room_id) = resolve_target(sessions, command.target).await?;
    require_manager(rooms, room_id, command.actor).await?;

    let link = InviteLink::issue(
        InviteCode::generate(),
        target,
        command.actor,
        command.expiry_days.unwrap_or(DEFAULT_EXPIRY_DAYS),
        now,
    )?;
    let stored = insert_with_unique_code(invites, link).await?;

    tracing::info!(
        target: "audit",
        action = "invite.create",
        actor_id = %command.actor,
        link_id = %stored.id,
        target_kind = stored.target.kind().as_str(),
        target_id = %stored.target.id(),
        expires_at = %stored.expires_at,
        outcome = "ok"
    );

    Ok(stored)
}

/// The target, plus the room whose management authorizes the operation.
pub(crate) async fn resolve_target<T>(
    sessions: &T,
    target: InviteTargetSelector,
) -> Result<(InviteTarget, RoomId), InviteOperationError>
where
    T: SessionStore + ?Sized,
{
    match target {
        InviteTargetSelector::Room(room_id) => Ok((InviteTarget::Room(room_id), room_id)),
        InviteTargetSelector::Session(session_id) => {
            let session = sessions
                .get(session_id)
                .await?
                .ok_or(InviteOperationError::NotFound)?;
            Ok((InviteTarget::Session(session_id), session.room_id))
        }
    }
}
