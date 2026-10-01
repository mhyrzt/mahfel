use mahfel_application::ports::{
    BoxFuture, FileStorage, FileStorageError, PresignedFileRequest, StoredFile,
};
use std::time::Duration;

use super::storage::S3FileStorage;

impl FileStorage for S3FileStorage {
    fn presign_upload<'a>(
        &'a self,
        key: &'a str,
        content_type: &'a str,
        size_bytes: u64,
        checksum_sha256: &'a str,
        expiry: Duration,
    ) -> BoxFuture<'a, Result<PresignedFileRequest, FileStorageError>> {
        super::presigned::upload(self, key, content_type, size_bytes, checksum_sha256, expiry)
    }

    fn read<'a>(
        &'a self,
        key: &'a str,
        max_bytes: u64,
    ) -> BoxFuture<'a, Result<StoredFile, FileStorageError>> {
        super::objects::read(self, key, max_bytes)
    }

    fn presign_download<'a>(
        &'a self,
        key: &'a str,
        file_name: &'a str,
        expiry: Duration,
    ) -> BoxFuture<'a, Result<PresignedFileRequest, FileStorageError>> {
        super::presigned::download(self, key, file_name, expiry)
    }

    fn presign_playback<'a>(
        &'a self,
        key: &'a str,
        expiry: Duration,
    ) -> BoxFuture<'a, Result<PresignedFileRequest, FileStorageError>> {
        super::presigned::playback(self, key, expiry)
    }

    fn delete<'a>(&'a self, key: &'a str) -> BoxFuture<'a, Result<(), FileStorageError>> {
        super::objects::delete(self, key)
    }

    fn promote<'a>(
        &'a self,
        from: &'a str,
        to: &'a str,
    ) -> BoxFuture<'a, Result<(), FileStorageError>> {
        super::objects::promote(self, from, to)
    }

    fn check_bucket(&self) -> BoxFuture<'_, Result<(), FileStorageError>> {
        super::objects::check_bucket(self)
    }
}
