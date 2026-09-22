//! Session authorization guards shared by the session use cases.

use crate::error::GuardError;
use crate::ports::{BlockStore, RoomStore};
use mahfel_domain::{ActorId, BlockScope, Room, RoomId, RoomStatus, RoomVisibility, SessionId};

/// Load the room and require the actor to be allowed to join it.
///
/// Deny-by-default: a manager or member may always join, and a non-member may
/// join only a `Public` room. Admission policy (lobby) is layered on top of
/// this by M4-06 through the same path.
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
