//! Authorization for operations that name a link directly.
//!
//! A link carries its own target, so a by-id operation authorizes against that
//! target rather than against where the caller happened to call from.

use super::error::InviteOperationError;
use crate::ports::{InviteStore, RoomStore, SessionStore};
use crate::rooms::require_manager;
use mahfel_domain::{ActorId, InviteLink, InviteLinkId, InviteTargetKind, RoomId, SessionId};

/// A target the caller cannot see yields `NotFound` so room existence cannot be
/// probed; a target they can see but do not manage yields `Forbidden`. Link ids
/// are unguessable, so the distinction exposes nothing.
///
/// # Errors
/// [`InviteOperationError::NotFound`] when the link or its target is missing or
/// invisible, [`InviteOperationError::Forbidden`] when unmanaged.
pub(crate) async fn require_managed_link<S, T, R>(
    invites: &S,
    sessions: &T,
    rooms: &R,
    actor: ActorId,
    link_id: InviteLinkId,
) -> Result<InviteLink, InviteOperationError>
where
    S: InviteStore + ?Sized,
    T: SessionStore + ?Sized,
    R: RoomStore + ?Sized,
{
    let link = invites
        .get(link_id)
        .await?
        .ok_or(InviteOperationError::NotFound)?;
    let room_id = target_room(link.target.kind(), link.target.id(), sessions).await?;
    require_manager(rooms, room_id, actor).await?;
    Ok(link)
}

/// A session that no longer exists has no authorizing room, so it fails closed.
async fn target_room<T>(
    kind: InviteTargetKind,
    target_id: uuid::Uuid,
    sessions: &T,
) -> Result<RoomId, InviteOperationError>
where
    T: SessionStore + ?Sized,
{
    match kind {
        InviteTargetKind::Room => Ok(RoomId::from_uuid(target_id)),
        InviteTargetKind::Session => {
            let session = sessions
                .get(SessionId::from_uuid(target_id))
                .await?
                .ok_or(InviteOperationError::NotFound)?;
            Ok(session.room_id)
        }
    }
}
