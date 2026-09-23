use crate::files::FileOperationError;
use crate::ports::{FileStore, RoomStore};
use mahfel_domain::{ActorId, FileContext, RoomRole, RoomStatus};

#[derive(Clone, Copy)]
pub enum FileAccess {
    Read,
    Upload,
    Delete,
}

pub(crate) async fn require_access<F: FileStore + ?Sized, R: RoomStore + ?Sized>(
    files: &F,
    rooms: &R,
    context: FileContext,
    actor: ActorId,
    access: FileAccess,
) -> Result<(), FileOperationError> {
    if let FileContext::Avatar(owner) = context {
        return if owner == actor {
            Ok(())
        } else {
            Err(FileOperationError::NotFound)
        };
    }
    let room_id = files
        .context_room(context)
        .await?
        .ok_or(FileOperationError::NotFound)?;
    let membership = rooms
        .get_member(room_id, actor)
        .await?
        .ok_or(FileOperationError::NotFound)?;
    if matches!(access, FileAccess::Upload)
        && rooms
            .get(room_id)
            .await?
            .is_none_or(|room| room.status != RoomStatus::Active)
    {
        return Err(FileOperationError::Forbidden);
    }
    if !permitted(context, membership.role, access) {
        return Err(FileOperationError::Forbidden);
    }
    Ok(())
}

fn permitted(context: FileContext, role: RoomRole, access: FileAccess) -> bool {
    role == RoomRole::Manager
        || (!matches!(context, FileContext::Recording(_)) && !matches!(access, FileAccess::Delete))
}

#[cfg(test)]
mod tests {
    use super::{FileAccess, permitted};
    use mahfel_domain::{FileContext, RoomId, RoomRole, SessionId};

    #[test]
    fn file_permission_matrix_is_deny_by_default_for_mutation() {
        let room = FileContext::Room(RoomId::new());
        let recording = FileContext::Recording(SessionId::new());
        assert!(permitted(room, RoomRole::Member, FileAccess::Read));
        assert!(permitted(room, RoomRole::Member, FileAccess::Upload));
        assert!(!permitted(room, RoomRole::Member, FileAccess::Delete));
        assert!(permitted(room, RoomRole::Manager, FileAccess::Delete));
        assert!(!permitted(recording, RoomRole::Member, FileAccess::Read));
        assert!(!permitted(recording, RoomRole::Member, FileAccess::Upload));
        assert!(permitted(recording, RoomRole::Manager, FileAccess::Read));
    }
}
