use super::RoomOperationError;
use crate::ports::{MediaGateway, RoomStore, StoreError};
use mahfel_domain::{ActorId, RoomId, RoomMember, RoomRole, ensure_member_removable};
use time::OffsetDateTime;

/// Manager-only: add a member, granting access immediately (auto join).
pub async fn add_member<R>(
    rooms: &R,
    room_id: RoomId,
    actor: ActorId,
    target: ActorId,
    role: RoomRole,
) -> Result<RoomMember, RoomOperationError>
where
    R: RoomStore + ?Sized,
{
    super::require_manager(rooms, room_id, actor).await?;
    let member = RoomMember::new(room_id, target, role, OffsetDateTime::now_utc());
    match rooms.add_member(&member).await {
        Ok(()) => Ok(member),
        Err(StoreError::Conflict(_)) => Err(RoomOperationError::AlreadyMember),
        // The target actor has no row: a client error, not an internal failure.
        Err(StoreError::MissingReference(_)) => Err(RoomOperationError::ActorNotFound),
        Err(error) => Err(error.into()),
    }
}

/// Manager-only: remove a membership, removing access. The last manager may
/// not be removed.
///
/// The membership change withdraws every media capability, so the reconcile
/// closes the target's producers: otherwise a principal removed mid-session
/// keeps publishing with a grant they no longer hold. A media failure is
/// reported rather than swallowed, because the membership is already gone and
/// an operator has to reconcile.
pub async fn remove_member<R, M>(
    rooms: &R,
    media: &M,
    room_id: RoomId,
    actor: ActorId,
    target: ActorId,
) -> Result<(), RoomOperationError>
where
    R: RoomStore + ?Sized,
    M: MediaGateway + ?Sized,
{
    super::require_manager(rooms, room_id, actor).await?;
    let members = rooms.list_members(room_id).await?;
    ensure_member_removable(&members, target)?;
    if !rooms.remove_member(room_id, target).await? {
        return Err(RoomOperationError::NotFound);
    }
    // Reconciled against the now-absent membership, so every source is closed.
    //
    // Best-effort for the response, loud in the log. The membership is already
    // gone, so failing the request would misreport a change that happened; but
    // a reconcile that fails means a removed principal may still be publishing,
    // which an operator has to know about rather than discover later.
    match crate::moderation::reconcile_media(rooms, media, target, room_id).await {
        Ok(_) => {}
        Err(error) => {
            tracing::error!(
                target: "audit",
                action = "room.member.remove.reconcile_failed",
                actor_id = %actor,
                room_id = %room_id,
                target_actor_id = %target,
                error = %error,
                outcome = "degraded"
            );
        }
    }
    tracing::info!(
        target: "audit",
        action = "room.member.remove",
        actor_id = %actor,
        room_id = %room_id,
        target_actor_id = %target,
        outcome = "ok"
    );
    Ok(())
}

/// Manager-only: change a member's role.
///
/// The last manager cannot be demoted, for the same reason it cannot be
/// removed: the room would be left with nobody who can manage it, which is the
/// invariant [`ensure_member_removable`] already names.
///
/// Demotion withdraws moderation but not participation, so the reconcile is a
/// no-op here: a member keeps `PublishMedia` and `ShareScreen`. It runs anyway,
/// so the rule holds if a role is ever introduced that lacks a media
/// capability.
pub async fn change_member_role<R, M>(
    rooms: &R,
    media: &M,
    room_id: RoomId,
    actor: ActorId,
    target: ActorId,
    role: RoomRole,
) -> Result<RoomMember, RoomOperationError>
where
    R: RoomStore + ?Sized,
    M: MediaGateway + ?Sized,
{
    super::require_manager(rooms, room_id, actor).await?;
    let members = rooms.list_members(room_id).await?;
    // Reuses the last-manager guard: promoting the last manager is fine, but
    // demoting them is not.
    if role != RoomRole::Manager {
        ensure_member_removable(&members, target)?;
    }
    let mut member = members
        .iter()
        .find(|member| member.actor_id == target)
        .cloned()
        .ok_or(RoomOperationError::NotFound)?;
    member.change_role(role, OffsetDateTime::now_utc());
    rooms
        .add_member(&member)
        .await
        .or_else(|error| match error {
            // Upsert semantics: adding an existing membership replaces its role.
            StoreError::Conflict(_) => Ok(()),
            other => Err(other),
        })?;

    // A demotion withdraws no media capability, so this is normally a no-op;
    // it is run anyway so the rule holds if a role is ever introduced that
    // lacks one. Best-effort for the same reason as removal.
    if let Err(error) = crate::moderation::reconcile_media(rooms, media, target, room_id).await {
        tracing::error!(
            target: "audit",
            action = "room.member.role_change.reconcile_failed",
            actor_id = %actor,
            room_id = %room_id,
            target_actor_id = %target,
            error = %error,
            outcome = "degraded"
        );
    }

    tracing::info!(
        target: "audit",
        action = "room.member.role_change",
        actor_id = %actor,
        room_id = %room_id,
        target_actor_id = %target,
        role = role.as_str(),
        outcome = "ok"
    );
    Ok(member)
}
