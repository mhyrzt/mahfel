//! Revoke an invite link. Revocation is terminal.

use super::error::InviteOperationError;
use super::guard;
use crate::ports::{InviteStore, RoomStore, SessionStore};
use mahfel_domain::{ActorId, InviteLink, InviteLinkId};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct RevokeInviteLink {
    pub actor: ActorId,
    pub link_id: InviteLinkId,
}

/// Authorization is derived from the link's own target, so a manager can only
/// revoke links belonging to a target they manage.
///
/// # Errors
/// [`InviteOperationError::NotFound`] when the link is missing or its target is
/// not visible to the caller, [`InviteOperationError::Revoked`] on a repeat.
pub async fn revoke_link<S, T, R>(
    invites: &S,
    sessions: &T,
    rooms: &R,
    command: RevokeInviteLink,
    now: OffsetDateTime,
) -> Result<InviteLink, InviteOperationError>
where
    S: InviteStore + ?Sized,
    T: SessionStore + ?Sized,
    R: RoomStore + ?Sized,
{
    let mut link =
        guard::require_managed_link(invites, sessions, rooms, command.actor, command.link_id)
            .await?;

    link.revoke(now)?;

    // Checked in memory first so a repeat is a clean conflict; the conditional
    // update then makes it true under concurrency.
    if !invites.revoke_if_active(command.link_id, now).await? {
        return Err(InviteOperationError::Revoked);
    }

    tracing::info!(
        target: "audit",
        action = "invite.revoke",
        actor_id = %command.actor,
        link_id = %command.link_id,
        outcome = "ok"
    );

    Ok(link)
}
