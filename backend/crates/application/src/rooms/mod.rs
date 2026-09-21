pub mod archive;
pub mod create;
pub mod get;
pub mod join;
pub mod list;
pub mod members;
pub mod update;

pub use archive::{ArchiveRoomError, archive_room};
pub use create::{CreateRoom, CreateRoomError, create_room};
pub use get::{GetRoomError, get_room};
pub use join::{JoinRoomError, join_room};
pub use list::{ListRoomsError, list_joined};
pub use members::{AddMemberError, RemoveMemberError, add_member, remove_member};
pub use update::{UpdateRoom, UpdateRoomError, update_settings};

use crate::ports::{RoomStore, StoreError};
use mahfel_domain::{ActorId, Room, RoomId, RoomRole};

/// Why a manager-only operation was rejected.
///
/// A missing room and a non-member caller collapse to `NotFound` so callers
/// cannot probe room existence; a member without the manager role is
/// `Forbidden`.
#[derive(Debug, thiserror::Error)]
pub(crate) enum ManagerGuardError {
    #[error("room not found")]
    NotFound,
    #[error("permission denied")]
    Forbidden,
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

/// Load the room and require the actor to be one of its managers.
pub(crate) async fn require_manager<R>(
    rooms: &R,
    room_id: RoomId,
    actor: ActorId,
) -> Result<Room, ManagerGuardError>
where
    R: RoomStore + ?Sized,
{
    let room = rooms
        .get(room_id)
        .await?
        .ok_or(ManagerGuardError::NotFound)?;
    let membership = rooms
        .get_member(room_id, actor)
        .await?
        .ok_or(ManagerGuardError::NotFound)?;
    if membership.role != RoomRole::Manager {
        return Err(ManagerGuardError::Forbidden);
    }
    Ok(room)
}
