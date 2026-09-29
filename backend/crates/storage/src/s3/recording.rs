use crate::S3FileStorage;
use aws_sdk_s3::primitives::ByteStream;
use mahfel_application::ports::{BoxFuture, FileStorageError, ObjectPage, RecordingStorage};
use std::path::Path;

impl RecordingStorage for S3FileStorage {
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
