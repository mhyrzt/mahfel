use crate::ports::{CollaborationAccess, CollaborationStore, StoreError};
use mahfel_domain::{ActorId, SessionId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CollaborationError {
    #[error("collaboration workspace unavailable")]
    Denied,
    #[error("collaboration workspace is read only")]
    ReadOnly,
    #[error(transparent)]
    Store(#[from] StoreError),
}

pub async fn require_read<S: CollaborationStore + ?Sized>(
    store: &S,
    session_id: SessionId,
    actor_id: ActorId,
) -> Result<CollaborationAccess, CollaborationError> {
    let access = store.access(session_id, actor_id).await?;
    if access.can_read {
        Ok(access)
    } else {
        Err(CollaborationError::Denied)
    }
}

pub async fn require_write<S: CollaborationStore + ?Sized>(
    store: &S,
    session_id: SessionId,
    actor_id: ActorId,
) -> Result<CollaborationAccess, CollaborationError> {
    let access = require_read(store, session_id, actor_id).await?;
    if access.live && access.can_edit {
        Ok(access)
    } else {
        Err(CollaborationError::ReadOnly)
    }
}
