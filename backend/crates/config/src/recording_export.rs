use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct RecordingExportSettings {
    pub renderer_url: String,
    pub renderer_api_key: String,
    pub scratch_directory: PathBuf,
    pub scratch_bytes: u64,
    pub output_bytes: u64,
    pub concurrency: usize,
    pub max_active_audio_tracks: usize,
    pub timeout_seconds: u64,
}
impl Default for RecordingExportSettings {
    fn default() -> Self {
        Self {
            renderer_url: "http://127.0.0.1:3000/api/internal/recording-render".into(),
            renderer_api_key: String::new(),
            scratch_directory: PathBuf::from("/tmp/mahfel-exports"),
            scratch_bytes: 20 * 1024 * 1024 * 1024,
            output_bytes: 8 * 1024 * 1024 * 1024,
            concurrency: 1,
            max_active_audio_tracks: 64,
            timeout_seconds: 86400,
        }
    }
}

impl std::fmt::Debug for RecordingExportSettings {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RecordingExportSettings")
            .field("renderer_url", &self.renderer_url)
            .field("renderer_api_key", &"[redacted]")
            .field("scratch_directory", &self.scratch_directory)
            .field("scratch_bytes", &self.scratch_bytes)
            .field("output_bytes", &self.output_bytes)
            .field("concurrency", &self.concurrency)
            .field("max_active_audio_tracks", &self.max_active_audio_tracks)
            .field("timeout_seconds", &self.timeout_seconds)
            .finish()
    }
}
