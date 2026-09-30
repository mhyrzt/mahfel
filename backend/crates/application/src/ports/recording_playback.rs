use crate::ports::{BoxFuture, StoreError};
use mahfel_domain::{ActorId, SessionId};
use uuid::Uuid;

pub trait RecordingPlaybackStore: Send + Sync {
    fn can_play(
        &self,
        session: SessionId,
        actor: ActorId,
    ) -> BoxFuture<'_, Result<bool, StoreError>>;
    fn manifest(
        &self,
        session: SessionId,
        recording: Uuid,
    ) -> BoxFuture<'_, Result<Option<String>, StoreError>>;
    fn delete(
        &self,
        session: SessionId,
        recording: Uuid,
        actor: ActorId,
    ) -> BoxFuture<'_, Result<(), StoreError>>;
}
