use aws_sdk_s3::presigning::PresigningConfig;
use mahfel_application::ports::{BoxFuture, FileStorageError, PresignedFileRequest};
use std::time::Duration;

use super::storage::S3FileStorage;

pub(super) fn upload<'a>(
    storage: &'a S3FileStorage,
    key: &'a str,
    content_type: &'a str,
    size_bytes: u64,
    checksum_sha256: &'a str,
    expiry: Duration,
) -> BoxFuture<'a, Result<PresignedFileRequest, FileStorageError>> {
    Box::pin(async move {
        let length = i64::try_from(size_bytes).map_err(|_| FileStorageError::TooLarge)?;
        let signed = storage
            .public
            .put_object()
            .bucket(&storage.bucket)
            .key(key)
            .content_type(content_type)
            .content_length(length)
            .checksum_sha256(checksum_sha256)
            .presigned(presign_config(expiry)?)
            .await
            .map_err(|_| FileStorageError::Unavailable)?;
        Ok(to_request(&signed))
    })
}

pub(super) fn download<'a>(
    storage: &'a S3FileStorage,
    key: &'a str,
    file_name: &'a str,
    expiry: Duration,
) -> BoxFuture<'a, Result<PresignedFileRequest, FileStorageError>> {
    Box::pin(async move {
        let safe_name: String = file_name
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_') {
                    character
                } else {
                    '_'
                }
            })
            .collect();
        let disposition = format!("attachment; filename=\"{safe_name}\"");
        let signed = storage
            .public
            .get_object()
            .bucket(&storage.bucket)
            .key(key)
            .response_content_disposition(disposition)
            .response_cache_control("private, no-store")
            .presigned(presign_config(expiry)?)
            .await
            .map_err(|_| FileStorageError::Unavailable)?;
        Ok(to_request(&signed))
    })
}

fn presign_config(expiry: Duration) -> Result<PresigningConfig, FileStorageError> {
    PresigningConfig::expires_in(expiry).map_err(|_| FileStorageError::Unavailable)
}

fn to_request(signed: &aws_sdk_s3::presigning::PresignedRequest) -> PresignedFileRequest {
    PresignedFileRequest {
        url: signed.uri().to_owned(),
        headers: signed
            .headers()
            .map(|(name, value)| (name.to_owned(), value.to_owned()))
            .collect(),
    }
}
