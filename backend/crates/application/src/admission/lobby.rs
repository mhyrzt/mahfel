//! The manager's view of who is waiting to join.

use super::error::AdmissionOperationError;
use crate::ports::{AdmissionStore, RoomStore, SessionStore};
use crate::rooms::require_manager;
use mahfel_domain::{
    ActorId, AdmissionOrigin, AdmissionRequest, AdmissionRequestId, RoomId, SessionId,
};
use time::OffsetDateTime;

const DEFAULT_LIMIT: u32 = 100;
/// A queue longer than the cap cannot exist, but the read stays bounded anyway.
const MAX_LIMIT: u32 = 200;

/// One waiting principal, as a manager sees them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LobbyEntry {
    pub request_id: AdmissionRequestId,
    pub actor_id: ActorId,
    pub origin: AdmissionOrigin,
    pub requested_at: OffsetDateTime,
}

/// A waiting set, for a session or for a room.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lobby {
    /// The session this queue belongs to, or `None` for a room-scoped queue.
    pub session_id: Option<SessionId>,
    pub entries: Vec<LobbyEntry>,
}

/// Manager-only: the principals waiting to join one session, oldest first.
///
/// # Errors
/// [`AdmissionOperationError::NotFound`] when the session is missing or
/// invisible, [`AdmissionOperationError::Forbidden`] when unmanaged.
pub async fn view_lobby<A, T, R>(
    admissions: &A,
    sessions: &T,
    rooms: &R,
    actor: ActorId,
    session_id: SessionId,
    limit: Option<u32>,
) -> Result<Lobby, AdmissionOperationError>
where
    A: AdmissionStore + ?Sized,
    T: SessionStore + ?Sized,
    R: RoomStore + ?Sized,
{
    let session = sessions
        .get(session_id)
        .await?
        .ok_or(AdmissionOperationError::NotFound)?;
    require_manager(rooms, session.room_id, actor).await?;

    let limit = resolve_limit(limit);
    let entries = admissions
        .list_pending(session_id, limit)
        .await?
        .into_iter()
        .map(LobbyEntry::from)
        .collect();

    Ok(Lobby {
        session_id: Some(session_id),
        entries,
    })
}

/// Manager-only: the principals waiting to join one room, oldest first.
///
/// These are requests that are not scoped to a single session, so a room carries
/// its own queue alongside each session's.
///
/// # Errors
/// [`AdmissionOperationError::NotFound`] when the room is missing or invisible,
/// [`AdmissionOperationError::Forbidden`] when unmanaged.
pub async fn view_room_lobby<A, R>(
    admissions: &A,
    rooms: &R,
    actor: ActorId,
    room_id: RoomId,
    limit: Option<u32>,
) -> Result<Lobby, AdmissionOperationError>
where
    A: AdmissionStore + ?Sized,
    R: RoomStore + ?Sized,
{
    require_manager(rooms, room_id, actor).await?;

    let entries = admissions
        .list_pending_for_room(room_id, resolve_limit(limit))
        .await?
        .into_iter()
        .map(LobbyEntry::from)
        .collect();

    Ok(Lobby {
        session_id: None,
        entries,
    })
}

fn resolve_limit(limit: Option<u32>) -> u32 {
    limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT)
}

impl From<AdmissionRequest> for LobbyEntry {
    fn from(request: AdmissionRequest) -> Self {
        Self {
            request_id: request.id,
            actor_id: request.actor_id,
            origin: request.origin,
            requested_at: request.requested_at,
        }
    }
}
