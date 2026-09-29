use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct RecordingSettings {
    pub spool_directory: PathBuf,
    pub recording_spool_bytes: u64,
    pub node_spool_bytes: u64,
    pub upload_concurrency: usize,
}
impl Default for RecordingSettings {
    fn default() -> Self {
        Self {
            spool_directory: PathBuf::from("/tmp/mahfel-recordings"),
            recording_spool_bytes: 268_435_456,
            node_spool_bytes: 1_073_741_824,
            upload_concurrency: 4,
        }
    }
}
