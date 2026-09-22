//! Read and change the admission policy on a room, and the access overrides on
//! a session.
//!
//! Management is capability-gated and audited. A session override may only
//! tighten access, which the domain enforces; these use cases supply the room a
//! session resolves against.

use super::error::AdmissionOperationError;
use crate::ports::{RoomStore, SessionStore};
use crate::rooms::require_manager;
use mahfel_domain::{
    ActorId, AdmissionPolicy, Room, RoomId, Session, SessionId, SessionVisibility,
};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct RoomAdmission {
    pub actor: ActorId,
    pub room_id: RoomId,
    pub admission: AdmissionPolicy,
}

/// Set a room's admission policy.
///
/// Existing memberships are unaffected; the policy applies to joins from now on.
///
/// # Errors
/// [`AdmissionOperationError::NotFound`] when the room is missing or invisible,
/// [`AdmissionOperationError::Forbidden`] when the caller does not manage it.
pub async fn set_room_admission<R>(
    rooms: &R,
    command: RoomAdmission,
    now: OffsetDateTime,
) -> Result<Room, AdmissionOperationError>
where
    R: RoomStore + ?Sized,
{
    let mut room = require_manager(rooms, command.room_id, command.actor).await?;
    room.set_admission(command.admission, now)?;
    rooms.save(&room).await?;

    tracing::info!(
        target: "audit",
        action = "admission.policy",
        actor_id = %command.actor,
        room_id = %command.room_id,
        admission = command.admission.as_str(),
        outcome = "ok"
    );
    Ok(room)
}

/// A session's own overrides, plus the values they resolve to.
#[derive(Debug, Clone, Copy)]
pub struct SessionAccess {
    pub actor: ActorId,
    pub session_id: SessionId,
    /// `None` clears the override back to inheritance.
    pub visibility: Option<SessionVisibility>,
    pub admission: Option<AdmissionPolicy>,
}

/// A session's effective access settings, resolved against its room.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionAccessView {
    pub visibility: SessionVisibility,
    pub admission: AdmissionPolicy,
    /// Whether each axis is overridden rather than inherited, so a client can
    /// show which one it changed.
    pub visibility_overridden: bool,
    pub admission_overridden: bool,
}

/// Change a session's access overrides.
///
/// # Errors
/// [`AdmissionOperationError::NotFound`] when the session is missing or
/// invisible, [`AdmissionOperationError::Forbidden`] when unmanaged,
/// [`AdmissionOperationError::Domain`] when an override would loosen access.
pub async fn set_session_access<T, R>(
    sessions: &T,
    rooms: &R,
    command: SessionAccess,
    now: OffsetDateTime,
) -> Result<SessionAccessView, AdmissionOperationError>
where
    T: SessionStore + ?Sized,
    R: RoomStore + ?Sized,
{
    let mut session = load_session(sessions, command.session_id).await?;
    let room = require_manager(rooms, session.room_id, command.actor).await?;

    session.set_visibility(command.visibility, &room, now)?;
    session.set_admission(command.admission, &room, now)?;
    sessions.save(&session).await?;

    tracing::info!(
        target: "audit",
        action = "admission.session_access",
        actor_id = %command.actor,
        session_id = %command.session_id,
        visibility = ?command.visibility.map(SessionVisibility::as_str),
        admission = ?command.admission.map(AdmissionPolicy::as_str),
        outcome = "ok"
    );

    Ok(view(&session, &room))
}

/// Read a session's effective access settings.
///
/// # Errors
/// [`AdmissionOperationError::NotFound`] when the session is missing or
/// invisible.
pub async fn view_session_access<T, R>(
    sessions: &T,
    rooms: &R,
    actor: ActorId,
    session_id: SessionId,
) -> Result<SessionAccessView, AdmissionOperationError>
where
    T: SessionStore + ?Sized,
    R: RoomStore + ?Sized,
{
    let session = load_session(sessions, session_id).await?;
    let room = require_manager(rooms, session.room_id, actor).await?;
    Ok(view(&session, &room))
}

fn view(session: &Session, room: &Room) -> SessionAccessView {
    SessionAccessView {
        visibility: session.effective_visibility(room),
        admission: session.effective_admission(room),
        visibility_overridden: session.visibility.is_some(),
        admission_overridden: session.admission.is_some(),
    }
}

async fn load_session<T>(
    sessions: &T,
    session_id: SessionId,
) -> Result<Session, AdmissionOperationError>
where
    T: SessionStore + ?Sized,
{
    sessions
        .get(session_id)
        .await?
        .ok_or(AdmissionOperationError::NotFound)
}
