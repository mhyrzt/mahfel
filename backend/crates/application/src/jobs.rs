//! Durable background-job contract shared by the API and worker.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

pub const RECORDING_FINALIZE_KIND: &str = "recordings.finalize";
pub const FILE_CLEANUP_KIND: &str = "files.cleanup_deleted_attachment";
pub const JOB_SCHEMA_VERSION: u16 = 1;
pub const MAX_ATTEMPTS: u16 = 10;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JobEnvelope {
    pub job_id: Uuid,
    pub job_type: String,
    pub version: u16,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    pub correlation_id: Option<String>,
    #[serde(default)]
    pub traceparent: Option<String>,
    #[serde(default)]
    pub tracestate: Option<String>,
    pub idempotency_key: String,
    pub generation: u32,
    pub attempt: u16,
    pub payload: Value,
}

impl JobEnvelope {
    #[must_use]
    pub fn recording_finalize(
        recording_id: Uuid,
        capture_generation: i32,
        now: OffsetDateTime,
    ) -> Self {
        let mut job = Self::file_cleanup(recording_id, now);
        job.job_type = RECORDING_FINALIZE_KIND.into();
        job.idempotency_key = format!("recording.finalize:{recording_id}:{capture_generation}");
        job.payload = serde_json::json!({"recording_id": recording_id, "capture_generation": capture_generation});
        job
    }

    #[must_use]
    pub fn file_cleanup(file_id: Uuid, now: OffsetDateTime) -> Self {
        Self {
            job_id: Uuid::new_v4(),
            job_type: FILE_CLEANUP_KIND.to_owned(),
            version: JOB_SCHEMA_VERSION,
            created_at: now,
            correlation_id: None,
            traceparent: None,
            tracestate: None,
            idempotency_key: format!("file.cleanup:{file_id}"),
            generation: 1,
            attempt: 1,
            payload: serde_json::json!({"file_id": file_id}),
        }
    }

    #[must_use]
    pub fn routing_key(&self) -> String {
        format!("{}.v{}", self.job_type, self.version)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FileCleanupPayload {
    pub file_id: Uuid,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JobFailure {
    Retryable(&'static str),
    Permanent(&'static str),
}

/// Delay before the next attempt. The first attempt has no delay.
#[must_use]
pub fn retry_delay_seconds(next_attempt: u16) -> Option<u64> {
    if !(2..=MAX_ATTEMPTS).contains(&next_attempt) {
        return None;
    }
    Some(
        30_u64
            .saturating_mul(1_u64 << u32::from(next_attempt - 2))
            .min(600),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_schedule_has_ten_total_attempts() {
        let delays: Vec<_> = (2..=10).map(retry_delay_seconds).collect();
        assert_eq!(
            delays,
            vec![
                Some(30),
                Some(60),
                Some(120),
                Some(240),
                Some(480),
                Some(600),
                Some(600),
                Some(600),
                Some(600)
            ]
        );
        assert_eq!(retry_delay_seconds(11), None);
    }

    #[test]
    fn file_cleanup_envelope_round_trips() {
        let job = JobEnvelope::file_cleanup(Uuid::new_v4(), OffsetDateTime::now_utc());
        let bytes = serde_json::to_vec(&job).expect("serialize job");
        let parsed: JobEnvelope = serde_json::from_slice(&bytes).expect("parse job");
        assert_eq!(parsed, job);
        assert_eq!(parsed.routing_key(), "files.cleanup_deleted_attachment.v1");
    }

    #[test]
    fn v1_fixture_tolerates_additive_fields() {
        let fixture = include_str!("../tests/fixtures/job-file-cleanup-v1.json");
        let parsed: JobEnvelope = serde_json::from_str(fixture).expect("compatible v1 fixture");
        let payload: FileCleanupPayload =
            serde_json::from_value(parsed.payload.clone()).expect("payload");
        assert_eq!(
            payload.file_id.to_string(),
            "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa"
        );
        assert_eq!(parsed.attempt, 1);
    }
}
