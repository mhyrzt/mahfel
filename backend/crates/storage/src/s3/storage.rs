use aws_sdk_s3::Client;
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
}
