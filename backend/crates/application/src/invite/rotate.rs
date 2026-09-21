//! Rotate an invite link's code.
//!
//! "Regenerate" and "rotate" are the same transition with different intent, so
//! there is one implementation: the previous code stops working the moment the
//! new one exists, and neither state is ever observable.

use super::error::InviteOperationError;
use super::guard;
use crate::ports::{InviteStore, RoomStore, SessionStore, StoreError};
use mahfel_domain::{ActorId, InviteCode, InviteLink, InviteLinkId, InviteLinkStatus};
use time::OffsetDateTime;

/// Bounded like creation: repeated collisions mean uniqueness is not behaving.
const MAX_ATTEMPTS: usize = 5;

#[derive(Debug, Clone)]
pub struct RotateInviteLink {
    pub actor: ActorId,
    pub link_id: InviteLinkId,
    /// `None` preserves the previous link's window rather than resetting it, so
    /// rotating a deliberately short link does not silently extend it.
    pub expiry_days: Option<i64>,
}

/// # Errors
/// [`InviteOperationError::NotFound`] when the link is missing or unmanaged,
/// [`InviteOperationError::InvalidExpiry`] when the lifetime is out of range,
/// [`InviteOperationError::CodeExhausted`] when every candidate collided.
pub async fn rotate_link<S, T, R>(
    invites: &S,
    sessions: &T,
    rooms: &R,
    command: RotateInviteLink,
    now: OffsetDateTime,
) -> Result<InviteLink, InviteOperationError>
where
    S: InviteStore + ?Sized,
    T: SessionStore + ?Sized,
    R: RoomStore + ?Sized,
{
    let previous =
        guard::require_managed_link(invites, sessions, rooms, command.actor, command.link_id)
            .await?;

    let expires_at = replacement_expiry(&previous, command.expiry_days, now)?;

    for _ in 0..MAX_ATTEMPTS {
        let replacement = InviteLink {
            id: InviteLinkId::new(),
            code: InviteCode::generate(),
            target: previous.target,
            status: InviteLinkStatus::Active,
            expires_at,
            created_by: previous.created_by,
            created_at: now,
            revoked_at: None,
        };

        match invites.rotate(previous.id, now, &replacement).await {
            Ok(()) => {
                tracing::info!(
                    target: "audit",
                    action = "invite.rotate",
                    actor_id = %command.actor,
                    previous_link_id = %previous.id,
                    replacement_link_id = %replacement.id,
                    target_kind = replacement.target.kind().as_str(),
                    target_id = %replacement.target.id(),
                    outcome = "ok"
                );
                return Ok(replacement);
            }
            // The transaction rolled back, so the previous code is still live.
            Err(StoreError::Conflict(_)) => continue,
            Err(other) => return Err(other.into()),
        }
    }
    Err(InviteOperationError::CodeExhausted)
}

fn replacement_expiry(
    previous: &InviteLink,
    expiry_days: Option<i64>,
    now: OffsetDateTime,
) -> Result<OffsetDateTime, InviteOperationError> {
    match expiry_days {
        Some(days) => {
            if days <= 0 || days > mahfel_domain::MAX_EXPIRY_DAYS {
                return Err(InviteOperationError::InvalidExpiry);
            }
            Ok(now + time::Duration::days(days))
        }
        None => Ok(now + (previous.expires_at - previous.created_at)),
    }
}
