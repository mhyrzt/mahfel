use crate::ports::{BoxFuture, FileStorageError};
use std::path::Path;

pub struct ObjectPage {
    pub keys: Vec<String>,
    pub continuation: Option<String>,
}

pub trait RecordingStorage: Send + Sync {
    fn put<'a>(
        &'a self,
        key: &'a str,
        bytes: Vec<u8>,
    ) -> BoxFuture<'a, Result<(), FileStorageError>>;
    fn put_fragment<'a>(
        &'a self,
        key: &'a str,
        path: &'a Path,
    ) -> BoxFuture<'a, Result<(), FileStorageError>>;
    fn read<'a>(
        &'a self,
        key: &'a str,
        maximum: usize,
    ) -> BoxFuture<'a, Result<Vec<u8>, FileStorageError>>;
    fn page<'a>(
        &'a self,
        prefix: &'a str,
        continuation: Option<String>,
    ) -> BoxFuture<'a, Result<ObjectPage, FileStorageError>>;
    fn copy<'a>(
        &'a self,
        source: &'a str,
        destination: &'a str,
    ) -> BoxFuture<'a, Result<(), FileStorageError>>;
}
