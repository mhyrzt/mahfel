use crate::{
    jobs::JobEnvelope,
    ports::{BoxFuture, StoreError},
};
use mahfel_domain::{ActorId, RecordingExport, SessionId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExportSource {
    pub export_id: Uuid,
    pub recording_id: Uuid,
    pub manifest_key: String,
    pub revision: i32,
}

#[derive(Clone, Debug)]
pub struct ExportArtifact {
    pub object_key: String,
    pub duration_ms: i64,
    pub size_bytes: i64,
    pub sha256: String,
}

pub trait RecordingExportStore: Send + Sync {
    fn can_download(
        &self,
        session: SessionId,
        actor: ActorId,
    ) -> BoxFuture<'_, Result<bool, StoreError>>;
    fn get(
        &self,
        session: SessionId,
        recording: Uuid,
    ) -> BoxFuture<'_, Result<Option<RecordingExport>, StoreError>>;
    fn retry(
        &self,
        session: SessionId,
        recording: Uuid,
    ) -> BoxFuture<'_, Result<RecordingExport, StoreError>>;
    fn begin<'a>(
        &'a self,
        job: &'a JobEnvelope,
        token: Uuid,
    ) -> BoxFuture<'a, Result<Option<ExportSource>, StoreError>>;
    fn accept<'a>(
        &'a self,
        job: &'a JobEnvelope,
        token: Uuid,
        artifact: &'a ExportArtifact,
    ) -> BoxFuture<'a, Result<bool, StoreError>>;
}

pub trait RecordingExporter: Send + Sync {
    fn discard<'a>(
        &'a self,
        _artifact: &'a ExportArtifact,
    ) -> BoxFuture<'a, Result<(), crate::jobs::JobFailure>> {
        Box::pin(async { Ok(()) })
    }
    fn export<'a>(
        &'a self,
        source: &'a ExportSource,
        token: Uuid,
    ) -> BoxFuture<'a, Result<ExportArtifact, crate::jobs::JobFailure>>;
}
