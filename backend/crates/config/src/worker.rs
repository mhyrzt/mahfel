use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct WorkerSettings {
    #[serde(default)]
    pub exports: crate::RecordingExportSettings,
    pub operations_bind: String,
    pub amqp_url: String,
    pub concurrency: usize,
    pub prefetch: u16,
    pub drain_timeout_seconds: u64,
}
