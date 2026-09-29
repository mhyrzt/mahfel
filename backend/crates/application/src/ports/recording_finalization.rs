use crate::{
    jobs::JobEnvelope,
    ports::{BoxFuture, StoreError},
};
use serde_json::Value;
use uuid::Uuid;

pub trait RecordingFinalizationStore: Send + Sync {
    fn import_receipt<'a>(
        &'a self,
        receipt: &'a crate::recording_artifacts::FragmentReceipt,
    ) -> BoxFuture<'a, Result<(), StoreError>>;
    fn metadata(
        &self,
        recording: Uuid,
        generation: i32,
    ) -> BoxFuture<'_, Result<Option<Value>, StoreError>>;
    fn page(
        &self,
        recording: Uuid,
        section: &'static str,
        offset: i64,
    ) -> BoxFuture<'_, Result<Vec<Value>, StoreError>>;
    fn accept<'a>(
        &'a self,
        job: &'a JobEnvelope,
        token: Uuid,
        manifest: &'a ManifestAcceptance,
    ) -> BoxFuture<'a, Result<bool, StoreError>>;
}

pub trait RecordingVerifier: Send + Sync {
    fn readable<'a>(
        &'a self,
        bytes: Vec<u8>,
        container: &'a str,
    ) -> BoxFuture<'a, Result<bool, StoreError>>;
}

pub struct ManifestAcceptance {
    pub recording: Uuid,
    pub generation: i32,
    pub key: String,
    pub partial: bool,
    pub usable: bool,
}
