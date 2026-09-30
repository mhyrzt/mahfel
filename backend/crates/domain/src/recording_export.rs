use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordingExportStatus {
    Queued,
    Rendering,
    Ready,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecordingExport {
    pub id: Uuid,
    pub recording_id: Uuid,
    pub status: RecordingExportStatus,
    pub progress: u16,
    pub duration_ms: Option<i64>,
    pub size_bytes: Option<i64>,
    pub failure_code: Option<String>,
    pub partial: bool,
    #[serde(skip)]
    pub object_key: Option<String>,
}
