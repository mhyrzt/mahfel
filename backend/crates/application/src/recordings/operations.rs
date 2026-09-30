use crate::{
    ApplicationError, ErrorKind,
    policy::ServerCapabilityResolver,
    ports::{
        Capability, CapabilityContext, CapabilityResolver, Principal, RecordingStore, RoomStore,
        SessionStore, StoreError,
    },
};
use mahfel_domain::{Recording, RecordingAction, SessionId};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RecordingOperationError {
    #[error("recording storage unavailable")]
    Unavailable,
    #[error("recording session not found")]
    NotFound,
    #[error("recording permission denied")]
    Forbidden,
    #[error("recording command conflicts with current state")]
    Conflict,
    #[error("recording storage unavailable: {0}")]
    Store(StoreError),
}

impl From<StoreError> for RecordingOperationError {
    fn from(error: StoreError) -> Self {
        match error {
            StoreError::Conflict(_) => Self::Conflict,
            StoreError::NotFound(_) | StoreError::MissingReference(_) => Self::NotFound,
            other => Self::Store(other),
        }
    }
}

impl ApplicationError for RecordingOperationError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::NotFound => ErrorKind::NotFound,
            Self::Forbidden => ErrorKind::Forbidden,
            Self::Conflict => ErrorKind::Conflict,
            Self::Unavailable => ErrorKind::Unavailable,
            Self::Store(_) => ErrorKind::Internal,
        }
    }
}

pub async fn list_recordings<S: RecordingStore + ?Sized>(
    store: &S,
    session_id: SessionId,
    principal: &Principal,
) -> Result<Vec<Recording>, RecordingOperationError> {
    if !principal
        .roles
        .iter()
        .any(|role| role.eq_ignore_ascii_case("admin"))
        && !store.can_access(session_id, principal.actor_id).await?
    {
        return Err(RecordingOperationError::NotFound);
    }
    Ok(store.list(session_id).await?)
}

pub async fn control_recording<S, R, T>(
    store: &S,
    rooms: &R,
    sessions: &T,
    session_id: SessionId,
    principal: &Principal,
    command_id: Uuid,
    action: RecordingAction,
) -> Result<Recording, RecordingOperationError>
where
    S: RecordingStore + ?Sized,
    R: RoomStore + ?Sized,
    T: SessionStore + ?Sized,
{
    let session = sessions
        .get(session_id)
        .await?
        .ok_or(RecordingOperationError::NotFound)?;
    let membership = rooms
        .get_member(session.room_id, principal.actor_id)
        .await?;
    let granted = ServerCapabilityResolver.resolve(
        principal,
        CapabilityContext::in_room(membership.map(|member| member.role)),
    );
    let admin = granted.has(Capability::ManageSystem);
    if !admin && !store.can_access(session_id, principal.actor_id).await? {
        return Err(RecordingOperationError::NotFound);
    }
    if !granted.has(Capability::ManageRecording) {
        return Err(RecordingOperationError::Forbidden);
    }
    Ok(store
        .command(session_id, principal.actor_id, command_id, action)
        .await?)
}
