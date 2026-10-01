use crate::ports::BoxFuture;
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum FileStorageError {
    #[error("object not found")]
    NotFound,
    #[error("object exceeds allowed size")]
    TooLarge,
    #[error("object storage unavailable")]
    Unavailable,
}

pub struct PresignedFileRequest {
    pub url: String,
    pub headers: Vec<(String, String)>,
}

pub struct StoredFile {
    pub bytes: Vec<u8>,
    pub content_type: Option<String>,
}

pub trait FileStorage: Send + Sync {
    fn presign_upload<'a>(
        &'a self,
        key: &'a str,
        content_type: &'a str,
        size_bytes: u64,
        checksum_sha256: &'a str,
        expiry: Duration,
    ) -> BoxFuture<'a, Result<PresignedFileRequest, FileStorageError>>;

    fn read<'a>(
        &'a self,
        key: &'a str,
        max_bytes: u64,
    ) -> BoxFuture<'a, Result<StoredFile, FileStorageError>>;

    fn presign_download<'a>(
        &'a self,
        key: &'a str,
        file_name: &'a str,
        expiry: Duration,
    ) -> BoxFuture<'a, Result<PresignedFileRequest, FileStorageError>>;

    fn presign_playback<'a>(
        &'a self,
        key: &'a str,
        expiry: Duration,
    ) -> BoxFuture<'a, Result<PresignedFileRequest, FileStorageError>> {
        self.presign_download(key, "segment.mp4", expiry)
    }

    fn delete<'a>(&'a self, key: &'a str) -> BoxFuture<'a, Result<(), FileStorageError>>;
    fn promote<'a>(
        &'a self,
        from: &'a str,
        to: &'a str,
    ) -> BoxFuture<'a, Result<(), FileStorageError>>;
    fn check_bucket(&self) -> BoxFuture<'_, Result<(), FileStorageError>>;
}
