//! Session authorization guards shared by the session use cases.

use crate::error::GuardError;
use crate::policy::ServerCapabilityResolver;
use crate::ports::{
    BlockStore, Capability, CapabilityContext, Principal, RoomStore, require_capability,
};
use mahfel_domain::{ActorId, BlockScope, Room, RoomId, RoomStatus, RoomVisibility, SessionId};

/// Load the room and require the actor to be allowed to join it.
///
/// Deny-by-default: a manager or member may always join, and a non-member may
/// join only a `Public` room. Admission policy is checked separately.
pub(crate) async fn require_joinable<R>(
    rooms: &R,
    room_id: RoomId,
    actor: ActorId,
) -> Result<Room, GuardError>
where
    R: RoomStore + ?Sized,
{
    let room = rooms.get(room_id).await?.ok_or(GuardError::NotFound)?;
    if room.status != RoomStatus::Active {
        return Err(GuardError::Forbidden);
    }
    if rooms.get_member(room_id, actor).await?.is_some() {
        return Ok(room);
    }
    if room.visibility == RoomVisibility::Public {
        return Ok(room);
    }
    Err(GuardError::Forbidden)
}

/// Resolve the room-scoped manage capability from the authenticated principal.
pub(crate) async fn require_manage_session<R>(
    rooms: &R,
    room_id: RoomId,
    principal: &Principal,
) -> Result<Room, GuardError>
where
    R: RoomStore + ?Sized,
{
    let room = rooms.get(room_id).await?.ok_or(GuardError::NotFound)?;
    let membership = rooms.get_member(room_id, principal.actor_id).await?;
    let role = membership.as_ref().map(|member| member.role);
    require_capability(
        &ServerCapabilityResolver,
        principal,
        CapabilityContext::in_room(role),
        Capability::ManageSession,
    )
    .map_err(|_| {
        if membership.is_some() {
            GuardError::Forbidden
        } else {
            GuardError::NotFound
        }
    })?;
    Ok(room)
}

/// Require that no active block bars the actor from this room or session.
///
/// The room check runs first because a room block already covers every session
/// in it, and nothing writes per-session rows for a room block. Both are
/// independent durable facts, so a block that predated the other survives its
/// lift. A denial is [`GuardError::Forbidden`]: the caller may see the session
/// and is deliberately barred from it.
///
/// Whether a block is still active is the store's concern, since an expired
/// block is inactive without ever being lifted.
pub(crate) async fn require_not_blocked<B>(
    blocks: &B,
    room_id: RoomId,
    session_id: SessionId,
    actor: ActorId,
) -> Result<(), GuardError>
where
    B: BlockStore + ?Sized,
{
    if blocks
        .get_active(BlockScope::Room(room_id), actor)
        .await?
        .is_some()
    {
        return Err(GuardError::Forbidden);
    }
    if blocks
        .get_active(BlockScope::Session(session_id), actor)
        .await?
        .is_some()
    {
        return Err(GuardError::Forbidden);
    }
    Ok(())
}
