use crate::ports::{BoxFuture, StoreError};
use mahfel_domain::{ActorId, Recording, RecordingAction, SessionId};
use uuid::Uuid;

pub trait RecordingStore: Send + Sync {
    fn expire_capture(&self, _session: SessionId) -> BoxFuture<'_, Result<(), StoreError>> {
        Box::pin(async { Ok(()) })
    }
    fn capture_pending(&self, _session: SessionId) -> BoxFuture<'_, Result<bool, StoreError>> {
        Box::pin(async { Ok(false) })
    }
    fn can_access(
        &self,
        session: SessionId,
        actor: ActorId,
    ) -> BoxFuture<'_, Result<bool, StoreError>>;
    fn list(&self, session: SessionId) -> BoxFuture<'_, Result<Vec<Recording>, StoreError>>;
    fn command(
        &self,
        session: SessionId,
        actor: ActorId,
        command_id: Uuid,
        action: RecordingAction,
    ) -> BoxFuture<'_, Result<Recording, StoreError>>;
}
