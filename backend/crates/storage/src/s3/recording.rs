use crate::S3FileStorage;
use aws_sdk_s3::primitives::ByteStream;
use mahfel_application::ports::{BoxFuture, FileStorageError, ObjectPage, RecordingStorage};
use std::path::Path;

impl RecordingStorage for S3FileStorage {
    fn download<'a>(
        &'a self,
        key: &'a str,
        path: &'a Path,
        maximum: u64,
    ) -> BoxFuture<'a, Result<u64, FileStorageError>> {
        Box::pin(async move {
            use tokio::io::AsyncReadExt;
            let _permit = self
                .reads
                .acquire()
                .await
                .map_err(|_| FileStorageError::Unavailable)?;
            let response = self
                .internal
                .get_object()
                .bucket(&self.bucket)
                .key(key)
                .send()
                .await
                .map_err(|_| FileStorageError::Unavailable)?;
            let length = response
                .content_length()
                .and_then(|size| u64::try_from(size).ok())
                .ok_or(FileStorageError::Unavailable)?;
            if length > maximum {
                return Err(FileStorageError::TooLarge);
            }
            let mut reader = response
                .body
                .into_async_read()
                .take(maximum.saturating_add(1));
            let mut file = tokio::fs::File::create(path)
                .await
                .map_err(|_| FileStorageError::Unavailable)?;
            let copied = tokio::io::copy(&mut reader, &mut file)
                .await
                .map_err(|_| FileStorageError::Unavailable)?;
            if copied > maximum || copied != length {
                return Err(FileStorageError::TooLarge);
            }
            Ok(copied)
        })
    }
    fn upload_export<'a>(
        &'a self,
        key: &'a str,
        path: &'a Path,
    ) -> BoxFuture<'a, Result<(), FileStorageError>> {
        Box::pin(async move {
            use tokio::io::AsyncReadExt;
            let upload = self
                .internal
                .create_multipart_upload()
                .bucket(&self.bucket)
                .key(key)
                .content_type("video/mp4")
                .send()
                .await
                .map_err(|_| FileStorageError::Unavailable)?;
            let upload_id = upload
                .upload_id()
                .ok_or(FileStorageError::Unavailable)?
                .to_owned();
            let guard = super::export_upload::MultipartGuard::new(
                self.internal.clone(),
                self.bucket.clone(),
                key.to_owned(),
                upload_id.clone(),
            );
            let mut file = tokio::fs::File::open(path)
                .await
                .map_err(|_| FileStorageError::Unavailable)?;
            let mut parts = Vec::new();
            for part_number in 1..=10000 {
                let mut bytes = Vec::with_capacity(16 * 1024 * 1024);
                (&mut file)
                    .take(16 * 1024 * 1024)
                    .read_to_end(&mut bytes)
                    .await
                    .map_err(|_| FileStorageError::Unavailable)?;
                if bytes.is_empty() {
                    break;
                }
                let response = self
                    .internal
                    .upload_part()
                    .bucket(&self.bucket)
                    .key(key)
                    .upload_id(&upload_id)
                    .part_number(part_number)
                    .body(ByteStream::from(bytes))
                    .send()
                    .await
                    .map_err(|_| FileStorageError::Unavailable)?;
                let etag = response.e_tag().ok_or(FileStorageError::Unavailable)?;
                parts.push(
                    aws_sdk_s3::types::CompletedPart::builder()
                        .part_number(part_number)
                        .e_tag(etag)
                        .build(),
                );
            }
            self.internal
                .complete_multipart_upload()
                .bucket(&self.bucket)
                .key(key)
                .upload_id(&upload_id)
                .multipart_upload(
                    aws_sdk_s3::types::CompletedMultipartUpload::builder()
                        .set_parts(Some(parts))
                        .build(),
                )
                .send()
                .await
                .map_err(|_| FileStorageError::Unavailable)?;
            guard.complete();
            Ok(())
        })
    }
    fn put<'a>(
        &'a self,
        key: &'a str,
        bytes: Vec<u8>,
    ) -> BoxFuture<'a, Result<(), FileStorageError>> {
        Box::pin(async move {
            self.internal
                .put_object()
                .bucket(&self.bucket)
                .key(key)
                .body(ByteStream::from(bytes))
                .send()
                .await
                .map_err(|_| FileStorageError::Unavailable)?;
            Ok(())
        })
    }
    fn put_fragment<'a>(
        &'a self,
        key: &'a str,
        path: &'a Path,
    ) -> BoxFuture<'a, Result<(), FileStorageError>> {
        Box::pin(async move {
            let body = ByteStream::from_path(path)
                .await
                .map_err(|_| FileStorageError::Unavailable)?;
            self.internal
                .put_object()
                .bucket(&self.bucket)
                .key(key)
                .body(body)
                .send()
                .await
                .map_err(|_| FileStorageError::Unavailable)?;
            Ok(())
        })
    }
    fn read<'a>(
        &'a self,
        key: &'a str,
        maximum: usize,
    ) -> BoxFuture<'a, Result<Vec<u8>, FileStorageError>> {
        Box::pin(async move {
            let _permit = self
                .reads
                .acquire()
                .await
                .map_err(|_| FileStorageError::Unavailable)?;
            let response = self
                .internal
                .get_object()
                .bucket(&self.bucket)
                .key(key)
                .range(format!("bytes=0-{maximum}"))
                .send()
                .await
                .map_err(|_| FileStorageError::Unavailable)?;
            if response
                .content_length()
                .is_none_or(|size| size < 0 || size as u64 > maximum as u64)
            {
                return Err(FileStorageError::Unavailable);
            }
            let bytes = response
                .body
                .collect()
                .await
                .map_err(|_| FileStorageError::Unavailable)?
                .into_bytes();
            if bytes.len() > maximum {
                return Err(FileStorageError::Unavailable);
            }
            Ok(bytes.to_vec())
        })
    }
    fn page<'a>(
        &'a self,
        prefix: &'a str,
        continuation: Option<String>,
    ) -> BoxFuture<'a, Result<ObjectPage, FileStorageError>> {
        Box::pin(async move {
            let response = self
                .internal
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(prefix)
                .max_keys(100)
                .set_continuation_token(continuation)
                .send()
                .await
                .map_err(|_| FileStorageError::Unavailable)?;
            Ok(ObjectPage {
                keys: response
                    .contents()
                    .iter()
                    .filter_map(|object| object.key().map(str::to_owned))
                    .collect(),
                continuation: response.next_continuation_token().map(str::to_owned),
            })
        })
    }
    fn copy<'a>(
        &'a self,
        source: &'a str,
        destination: &'a str,
    ) -> BoxFuture<'a, Result<(), FileStorageError>> {
        Box::pin(async move {
            // Application file keys contain only generated UUIDs and fixed ASCII components.
            self.internal
                .copy_object()
                .bucket(&self.bucket)
                .key(destination)
                .copy_source(format!("{}/{}", self.bucket, source))
                .send()
                .await
                .map_err(|_| FileStorageError::Unavailable)?;
            Ok(())
        })
    }
}
