use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct RecordingExportSettings {
    pub renderer_directory: PathBuf,
    pub scratch_directory: PathBuf,
    pub scratch_bytes: u64,
    pub output_bytes: u64,
    pub concurrency: usize,
    pub max_active_audio_tracks: usize,
    pub timeout_seconds: u64,
    pub bun: PathBuf,
}
impl Default for RecordingExportSettings {
    fn default() -> Self {
        Self {
            renderer_directory: PathBuf::from("../frontend/recording-renderer-dist"),
            scratch_directory: PathBuf::from("/tmp/mahfel-exports"),
            scratch_bytes: 20 * 1024 * 1024 * 1024,
            output_bytes: 8 * 1024 * 1024 * 1024,
            concurrency: 1,
            max_active_audio_tracks: 64,
            timeout_seconds: 86400,
            bun: PathBuf::from("bun"),
        }
    }
}
