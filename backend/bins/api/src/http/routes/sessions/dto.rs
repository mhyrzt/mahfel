use mahfel_application::ports::CapabilitySet;
use mahfel_domain::{Session, SessionStatus};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;

/// Transport-level session status schema. Kept out of the domain so `utoipa`
/// never reaches `mahfel-domain` (ADR 0015).
#[derive(Clone, Copy, Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SessionStatusDto {
    Scheduled,
    Live,
    Ended,
    Cancelled,
}

impl From<SessionStatus> for SessionStatusDto {
    fn from(value: SessionStatus) -> Self {
        match value {
            SessionStatus::Scheduled => Self::Scheduled,
            SessionStatus::Live => Self::Live,
            SessionStatus::Ended => Self::Ended,
            SessionStatus::Cancelled => Self::Cancelled,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateSessionRequest {
    /// Optional advisory start instant. A session may also be ad-hoc.
    #[serde(default)]
    pub(crate) scheduled_start_at: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct SessionResponse {
    pub(crate) id: String,
    pub(crate) room_id: String,
    pub(crate) status: SessionStatusDto,
    pub(crate) scheduled_start_at: Option<OffsetDateTime>,
    pub(crate) started_at: Option<OffsetDateTime>,
    pub(crate) ended_at: Option<OffsetDateTime>,
    pub(crate) created_at: OffsetDateTime,
    pub(crate) updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct SessionCapabilitiesResponse {
    /// Capability names the caller holds in the session's room. Absent means
    /// denied, so an empty list is a complete answer.
    pub(crate) capabilities: Vec<String>,
}

impl SessionCapabilitiesResponse {
    pub(crate) fn from_set(capabilities: &CapabilitySet) -> Self {
        Self {
            capabilities: capabilities
                .iter()
                .map(|capability| capability.as_str().to_owned())
                .collect(),
        }
    }
}

impl From<Session> for SessionResponse {
    fn from(session: Session) -> Self {
        Self {
            id: session.id.to_string(),
            room_id: session.room_id.to_string(),
            status: session.status.into(),
            scheduled_start_at: session.scheduled_start_at,
            started_at: session.started_at,
            ended_at: session.ended_at,
            created_at: session.created_at,
            updated_at: session.updated_at,
        }
    }
}
