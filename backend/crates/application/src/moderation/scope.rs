//! Resolving a block's scope to the room that authorizes it.

use super::error::ModerationError;
use crate::ports::{RoomStore, SessionStore};
use mahfel_domain::{BlockScope, RoomId};

/// The room a scope is authorized against.
///
/// A room scope names its room directly. A session scope resolves through its
/// session, so a caller cannot block someone from a session in a room they do
/// not manage: the room that comes back is the session's own.
///
/// # Errors
/// [`ModerationError::NotFound`] when the target does not exist, which is
/// deliberate: a caller must not learn whether a session exists in a room they
/// cannot see.
pub(crate) async fn authorize_scope<R, T>(
    rooms: &R,
    sessions: &T,
    scope: &BlockScope,
) -> Result<RoomId, ModerationError>
where
    R: RoomStore + ?Sized,
    T: SessionStore + ?Sized,
{
    match scope {
        BlockScope::Room(room_id) => {
            rooms
                .get(*room_id)
                .await?
                .ok_or(ModerationError::NotFound)?;
            Ok(*room_id)
        }
        BlockScope::Session(session_id) => {
            let session = sessions
                .get(*session_id)
                .await?
                .ok_or(ModerationError::NotFound)?;
            Ok(session.room_id)
        }
    }
}
