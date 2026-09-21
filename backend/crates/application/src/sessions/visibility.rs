use super::error::SessionOperationError;
use crate::ports::{RoomStore, SessionStore};
use mahfel_domain::{ActorId, Session, SessionId};

#[derive(Debug, Clone)]
pub struct GetSessionForCaller {
    pub actor: ActorId,
    pub session_id: SessionId,
}

/// Load one session for a caller who may join it.
///
/// Visibility is the join rule: a manager, a member, and a non-member of a
/// public room may load the session, matching [`require_joinable`](super::require_joinable).
/// A caller without access gets [`SessionOperationError::NotFound`] so the
/// session's existence cannot be probed. Strictly manager-scoped reads keep
/// using [`get_session`](super::get_session).
pub async fn get_session_for_caller<R, S>(
    sessions: &R,
    rooms: &S,
    command: GetSessionForCaller,
) -> Result<Session, SessionOperationError>
where
    R: SessionStore + ?Sized,
    S: RoomStore + ?Sized,
{
    let session = sessions
        .get(command.session_id)
        .await?
        .ok_or(SessionOperationError::NotFound)?;
    // Collapse "exists but not visible" and "does not exist" into one variant
    // so the response cannot be used to probe for a private session.
    super::require_joinable(rooms, session.room_id, command.actor)
        .await
        .map_err(|_| SessionOperationError::NotFound)?;
    Ok(session)
}
