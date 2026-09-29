use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FragmentReceipt {
    pub recording_id: Uuid,
    pub generation: u32,
    pub boot_id: Uuid,
    pub producer_id: String,
    pub actor_id: Uuid,
    pub source: String,
    pub codec: String,
    pub container: String,
    pub unix_origin_ns: u64,
    pub start_ns: u64,
    pub end_ns: u64,
    pub object_key: String,
    pub sha256: String,
    pub bytes: u64,
}
