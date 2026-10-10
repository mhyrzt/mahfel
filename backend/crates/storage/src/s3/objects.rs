use mahfel_application::ports::{BoxFuture, FileStorageError, StoredFile};

use super::storage::S3FileStorage;

pub(super) fn read<'a>(
    storage: &'a S3FileStorage,
    key: &'a str,
    max_bytes: u64,
) -> BoxFuture<'a, Result<StoredFile, FileStorageError>> {
    Box::pin(async move {
        let _permit = storage
            .reads
            .acquire()
            .await
            .map_err(|_| FileStorageError::Unavailable)?;
        let head = storage
            .internal
            .head_object()
            .bucket(&storage.bucket)
            .key(key)
            .send()
            .await
            .map_err(|error| {
                if error
                    .as_service_error()
                    .is_some_and(|service| service.is_not_found())
                {
                    FileStorageError::NotFound
                } else {
                    FileStorageError::Unavailable
                }
            })?;
        let length = head.content_length().ok_or(FileStorageError::Unavailable)?;
        if length < 0 || length as u64 > max_bytes {
            return Err(FileStorageError::TooLarge);
        }
        let object = storage
            .internal
            .get_object()
            .bucket(&storage.bucket)
            .key(key)
            .range(format!("bytes=0-{max_bytes}"))
            .send()
            .await
            .map_err(|_| FileStorageError::Unavailable)?;
        let content_type = object.content_type().map(str::to_owned);
        let body = object
            .body
            .collect()
            .await
            .map_err(|_| FileStorageError::Unavailable)?
            .into_bytes();
        if body.len() as u64 > max_bytes {
            return Err(FileStorageError::TooLarge);
        }
        Ok(StoredFile {
            bytes: body.to_vec(),
            content_type,
        })
    })
}

pub(super) fn delete<'a>(
    storage: &'a S3FileStorage,
    key: &'a str,
) -> BoxFuture<'a, Result<(), FileStorageError>> {
    Box::pin(async move {
        storage
            .internal
            .delete_object()
            .bucket(&storage.bucket)
            .key(key)
            .send()
            .await
            .map_err(|_| FileStorageError::Unavailable)?;
        Ok(())
    })
}

pub(super) fn promote<'a>(
    storage: &'a S3FileStorage,
    from: &'a str,
    to: &'a str,
) -> BoxFuture<'a, Result<(), FileStorageError>> {
    Box::pin(async move {
        let source = storage
            .internal
            .head_object()
            .bucket(&storage.bucket)
            .key(from)
            .send()
            .await
            .map_err(|_| FileStorageError::Unavailable)?;
        let expires = source
            .expires_string
            .as_deref()
            .map(|value| {
                aws_sdk_s3::primitives::DateTime::from_str(
                    value,
                    aws_sdk_s3::primitives::DateTimeFormat::HttpDate,
                )
            })
            .transpose()
            .map_err(|_| FileStorageError::Unavailable)?;
        storage
            .internal
            .copy_object()
            .bucket(&storage.bucket)
            .key(to)
            .copy_source(format!("{}/{from}", storage.bucket))
            // RustFS does not reliably retain Content-Type with implicit/COPY metadata.
            .metadata_directive(aws_sdk_s3::types::MetadataDirective::Replace)
            .set_content_type(source.content_type)
            .set_cache_control(source.cache_control)
            .set_content_disposition(source.content_disposition)
            .set_content_encoding(source.content_encoding)
            .set_content_language(source.content_language)
            .set_expires(expires)
            .set_metadata(source.metadata)
            .send()
            .await
            .map_err(|_| FileStorageError::Unavailable)?;
        Ok(())
    })
}

pub(super) fn check_bucket(storage: &S3FileStorage) -> BoxFuture<'_, Result<(), FileStorageError>> {
    Box::pin(async move {
        storage
            .internal
            .head_bucket()
            .bucket(&storage.bucket)
            .send()
            .await
            .map_err(|_| FileStorageError::Unavailable)?;
        Ok(())
    })
}
