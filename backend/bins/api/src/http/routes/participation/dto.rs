use mahfel_application::participation::{ParticipantSummary, SessionSummary};
use mahfel_domain::{CorrectionReason, ParticipationInterval};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;

/// Transport-level correction reason. Kept out of the domain so `utoipa` never
/// reaches `mahfel-domain` (ADR 0015).
#[derive(Clone, Copy, Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CorrectionReasonDto {
    WrongEnd,
    WrongStart,
    Erroneous,
    ManualAdjustment,
}

impl From<CorrectionReasonDto> for CorrectionReason {
    fn from(value: CorrectionReasonDto) -> Self {
        match value {
            CorrectionReasonDto::WrongEnd => Self::WrongEnd,
            CorrectionReasonDto::WrongStart => Self::WrongStart,
            CorrectionReasonDto::Erroneous => Self::Erroneous,
            CorrectionReasonDto::ManualAdjustment => Self::ManualAdjustment,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CorrectIntervalRequest {
    /// Start of the replacement interval, as an RFC 3339 timestamp.
    #[serde(with = "super::super::rfc3339")]
    pub(crate) started_at: OffsetDateTime,
    /// End of the replacement interval, as an RFC 3339 timestamp. Omit to
    /// record the participant as still connected.
    #[serde(default, with = "super::super::rfc3339::option")]
    pub(crate) ended_at: Option<OffsetDateTime>,
    pub(crate) reason: CorrectionReasonDto,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct IntervalResponse {
    pub(crate) id: String,
    pub(crate) participant_id: String,
    pub(crate) session_id: String,
    pub(crate) actor_id: String,
    #[serde(with = "super::super::rfc3339")]
    pub(crate) started_at: OffsetDateTime,
    #[serde(with = "super::super::rfc3339::option")]
    pub(crate) ended_at: Option<OffsetDateTime>,
    /// Present when a correction replaced this interval. The row is retained.
    pub(crate) superseded_by: Option<String>,
    pub(crate) correction_reason: Option<String>,
    /// Connected seconds this interval contributes, or the elapsed time when it
    /// is still open and no session end has stopped the clock.
    pub(crate) connected_seconds: i64,
}

impl IntervalResponse {
    pub(crate) fn from_interval(interval: &ParticipationInterval, now: OffsetDateTime) -> Self {
        Self {
            id: interval.id.to_string(),
            participant_id: interval.participant_id.to_string(),
            session_id: interval.session_id.to_string(),
            actor_id: interval.actor_id.to_string(),
            started_at: interval.started_at,
            ended_at: interval.ended_at,
            superseded_by: interval.superseded_by.map(|id| id.to_string()),
            correction_reason: interval
                .correction_reason
                .map(|reason| reason.as_str().to_owned()),
            connected_seconds: interval.duration_seconds(now),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ParticipantParticipationResponse {
    pub(crate) session_id: String,
    pub(crate) participant_id: String,
    /// Total non-superseded connected seconds.
    pub(crate) connected_seconds: i64,
    /// Every interval, including superseded ones, so a reader sees the history.
    pub(crate) intervals: Vec<IntervalResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ParticipantSummaryResponse {
    pub(crate) participant_id: String,
    pub(crate) actor_id: String,
    pub(crate) connected_seconds: i64,
    /// Intervals replaced by a correction, so a reader knows the total changed.
    pub(crate) corrected_intervals: u32,
}

impl From<ParticipantSummary> for ParticipantSummaryResponse {
    fn from(value: ParticipantSummary) -> Self {
        Self {
            participant_id: value.participant_id.to_string(),
            actor_id: value.actor_id.to_string(),
            connected_seconds: value.connected_seconds,
            corrected_intervals: value.corrected_intervals,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct SessionParticipationResponse {
    pub(crate) session_id: String,
    pub(crate) participants: Vec<ParticipantSummaryResponse>,
}

impl From<SessionSummary> for SessionParticipationResponse {
    fn from(value: SessionSummary) -> Self {
        Self {
            session_id: value.session_id.to_string(),
            participants: value
                .participants
                .into_iter()
                .map(ParticipantSummaryResponse::from)
                .collect(),
        }
    }
}
