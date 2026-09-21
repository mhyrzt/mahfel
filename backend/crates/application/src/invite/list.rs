//! List a target's invite links.
//!
//! Manager-only. Revoked and expired links are included: a manager reviewing a
//! target needs to see what was issued and what was retracted, not only what is
//! live.

use super::create::{InviteTargetSelector, resolve_target};
use super::error::InviteOperationError;
use crate::ports::{InviteStore, RoomStore, SessionStore};
use crate::rooms::require_manager;
use mahfel_domain::{ActorId, InviteLink};
use time::OffsetDateTime;

const DEFAULT_LIMIT: u32 = 50;
/// Ceiling on one page, so a listing stays bounded regardless of history.
const MAX_LIMIT: u32 = 200;

#[derive(Debug, Clone)]
pub struct InviteLinkQuery {
    pub actor: ActorId,
    pub target: InviteTargetSelector,
    /// Clamped to `[1, MAX_LIMIT]`.
    pub limit: Option<u32>,
}

/// # Errors
/// [`InviteOperationError::NotFound`] when the target is missing or unmanaged.
pub async fn list_links<S, T, R>(
    invites: &S,
    sessions: &T,
    rooms: &R,
    command: InviteLinkQuery,
    now: OffsetDateTime,
) -> Result<Vec<InviteLink>, InviteOperationError>
where
    S: InviteStore + ?Sized,
    T: SessionStore + ?Sized,
    R: RoomStore + ?Sized,
{
    // Expiry is derived rather than stored, so the query itself is time-free;
    // the clock is part of the signature to match the other management commands.
    let _ = now;
    let (target, room_id) = resolve_target(sessions, command.target).await?;
    require_manager(rooms, room_id, command.actor).await?;

    let limit = command.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    Ok(invites
        .list_for_target(target.kind(), target.id(), limit)
        .await?)
}
