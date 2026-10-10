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

/// Roster identities are visible to admitted participants and moderators only.
/// Public discovery and self-scoped admission reads do not grant roster access.
pub async fn get_session_for_roster<R, S, P, B>(
    sessions: &R,
    rooms: &S,
    participants: &P,
    blocks: &B,
    principal: &crate::ports::Principal,
    session_id: SessionId,
) -> Result<Session, SessionOperationError>
where
    R: SessionStore + ?Sized,
    S: RoomStore + ?Sized,
    P: crate::ports::ParticipantStore + ?Sized,
    B: crate::ports::BlockStore + ?Sized,
{
    use crate::ports::{Capability, CapabilityContext, CapabilityResolver};
    let session = sessions
        .get(session_id)
        .await?
        .ok_or(SessionOperationError::NotFound)?;
    let room = rooms
        .get(session.room_id)
        .await?
        .ok_or(SessionOperationError::NotFound)?;
    if room.status != mahfel_domain::RoomStatus::Active {
        return Err(SessionOperationError::NotFound);
    }
    super::require_not_blocked(blocks, room.id, session_id, principal.actor_id)
        .await
        .map_err(|_| SessionOperationError::NotFound)?;
    let membership = rooms.get_member(room.id, principal.actor_id).await?;
    let capabilities = crate::policy::ServerCapabilityResolver.resolve(
        principal,
        CapabilityContext::in_room(membership.map(|member| member.role)),
    );
    if capabilities.has(Capability::MuteOthers)
        || capabilities.has(Capability::ManageSession)
        || participants
            .get_active(session_id, principal.actor_id)
            .await?
            .is_some()
    {
        return Ok(session);
    }
    Err(SessionOperationError::NotFound)
}
