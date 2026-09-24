use mahfel_application::jobs::JobEnvelope;
use time::OffsetDateTime;
use uuid::Uuid;

pub struct LeasedOutbox {
    pub id: Uuid,
    pub token: Uuid,
    pub envelope: JobEnvelope,
}

pub enum JobClaim {
    Claimed(Uuid),
    Busy,
    Stale,
    Dead,
    Terminal,
}

pub struct JobRecord {
    pub envelope: JobEnvelope,
    pub status: String,
    pub generation: i32,
    pub last_failure_code: Option<String>,
}

pub struct JobAttempt {
    pub generation: i32,
    pub attempt: i16,
    pub outcome: String,
    pub failure_code: Option<String>,
    pub finished_at: OffsetDateTime,
}

pub struct JobReplay {
    pub generation: i32,
    pub replayed_at: OffsetDateTime,
}

pub(super) type OutboxRow = (
    Uuid,
    Uuid,
    i32,
    i16,
    String,
    i16,
    serde_json::Value,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    OffsetDateTime,
);

pub(super) type StoredJobRow = (
    Uuid,
    String,
    i16,
    serde_json::Value,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    OffsetDateTime,
    i16,
    String,
    i32,
    Option<String>,
);
