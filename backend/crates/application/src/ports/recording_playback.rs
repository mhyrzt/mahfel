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
    fn playback_index(
        &self,
        session: SessionId,
        recording: Uuid,
    ) -> BoxFuture<'_, Result<Option<String>, StoreError>> {
        Box::pin(async move {
            let _ = (session, recording);
            Ok(None)
        })
    }
    fn delete(
        &self,
        session: SessionId,
        recording: Uuid,
        actor: ActorId,
    ) -> BoxFuture<'_, Result<(), StoreError>>;
    fn retry_processing(
        &self,
        session: SessionId,
        recording: Uuid,
    ) -> BoxFuture<'_, Result<bool, StoreError>> {
        Box::pin(async move {
            let _ = (session, recording);
            Ok(false)
        })
    }
}
