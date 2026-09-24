use aws_sdk_s3::{Client, primitives::ByteStream};
use mahfel_application::ports::FileStorageError;
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::S3Settings;

#[derive(Clone)]
pub struct S3FileStorage {
    pub(super) internal: Client,
    pub(super) public: Client,
    pub(super) bucket: String,
    pub(super) reads: Arc<Semaphore>,
}

impl S3FileStorage {
    #[must_use]
    pub fn new(settings: S3Settings) -> Self {
        let internal = super::client::build(&settings, &settings.internal_endpoint);
        let public = super::client::build(&settings, &settings.public_endpoint);

        Self {
            internal,
            public,
            bucket: settings.bucket,
            reads: Arc::new(Semaphore::new(4)),
        }
    }

    /// Upload a local development fixture through the internal S3 endpoint.
    /// The normal browser upload contract remains presigned and direct.
    pub async fn put_seed_object(
        &self,
        key: &str,
        content_type: &str,
        checksum_sha256: &str,
        bytes: Vec<u8>,
    ) -> Result<(), FileStorageError> {
        self.internal
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .checksum_sha256(checksum_sha256)
            .body(ByteStream::from(bytes))
            .send()
            .await
            .map_err(|_| FileStorageError::Unavailable)?;
        Ok(())
    }
}
