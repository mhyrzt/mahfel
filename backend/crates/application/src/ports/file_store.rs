use crate::ports::{BoxFuture, StoreError};
use mahfel_domain::{ActorId, FileContext, FileId, FileObject, FileStatus, RoomId, SessionId};
use time::OffsetDateTime;

pub trait FileStore: Send + Sync {
    fn context_room(
        &self,
        context: FileContext,
    ) -> BoxFuture<'_, Result<Option<RoomId>, StoreError>>;
    fn insert<'a>(&'a self, file: &'a FileObject) -> BoxFuture<'a, Result<(), StoreError>>;
    fn get(&self, id: FileId) -> BoxFuture<'_, Result<Option<FileObject>, StoreError>>;
    fn list(
        &self,
        context: FileContext,
        limit: u32,
        offset: u64,
    ) -> BoxFuture<'_, Result<Vec<FileObject>, StoreError>>;
    fn link_room_file_to_session(
        &self,
        file_id: FileId,
        session_id: SessionId,
    ) -> BoxFuture<'_, Result<bool, StoreError>>;
    fn transition<'a>(
        &'a self,
        id: FileId,
        from: FileStatus,
        to: FileStatus,
        verified_mime: Option<&'a str>,
        now: OffsetDateTime,
    ) -> BoxFuture<'a, Result<bool, StoreError>>;
    fn pending_before(
        &self,
        before: OffsetDateTime,
        limit: u32,
    ) -> BoxFuture<'_, Result<Vec<FileObject>, StoreError>>;
    fn remove(&self, id: FileId) -> BoxFuture<'_, Result<(), StoreError>>;
    fn publish<'a>(
        &'a self,
        id: FileId,
        key: &'a str,
        mime: &'a str,
        now: OffsetDateTime,
    ) -> BoxFuture<'a, Result<bool, StoreError>>;
    fn record_download(
        &self,
        id: FileId,
        actor: ActorId,
        now: OffsetDateTime,
    ) -> BoxFuture<'_, Result<(), StoreError>>;
}
