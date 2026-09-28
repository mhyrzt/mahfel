use mahfel_application::ports::CapabilitySet;
use mahfel_domain::{Session, SessionStatus, SessionVisibility};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;

pub(crate) use super::super::rooms::dto::AdmissionPolicyDto;

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

/// Transport-level visibility schema for a session override.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SessionVisibilityDto {
    Public,
    InviteOnly,
}

impl From<SessionVisibilityDto> for SessionVisibility {
    fn from(value: SessionVisibilityDto) -> Self {
        match value {
            SessionVisibilityDto::Public => Self::Public,
            SessionVisibilityDto::InviteOnly => Self::InviteOnly,
        }
    }
}

impl From<SessionVisibility> for SessionVisibilityDto {
    fn from(value: SessionVisibility) -> Self {
        match value {
            SessionVisibility::Public => Self::Public,
            SessionVisibility::InviteOnly => Self::InviteOnly,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateSessionRequest {
    /// Optional custom title. Otherwise the room's current title is displayed.
    #[schema(min_length = 1, max_length = 300)]
    pub(crate) title: Option<String>,
    /// Optional advisory start instant. A session may also be ad-hoc.
    #[serde(default, with = "super::super::rfc3339::option")]
    pub(crate) scheduled_start_at: Option<OffsetDateTime>,
    /// Optional advisory length in minutes, from 1 through 10080.
    #[schema(minimum = 1, maximum = 10080)]
    pub(crate) scheduled_duration_minutes: Option<u32>,
}

fn nullable_patch<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

fn nullable_rfc3339_patch<'de, D>(
    deserializer: D,
) -> Result<Option<Option<OffsetDateTime>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    super::super::rfc3339::option::deserialize(deserializer).map(Some)
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct UpdateSessionDetailsRequest {
    /// Omit to keep, null to clear, or provide a custom title.
    #[serde(default, deserialize_with = "nullable_patch")]
    #[schema(min_length = 1, max_length = 300)]
    pub(crate) title: Option<Option<String>>,
    /// Omit to keep, null to clear, or provide an RFC3339 instant.
    #[serde(default, deserialize_with = "nullable_rfc3339_patch")]
    pub(crate) scheduled_start_at: Option<Option<OffsetDateTime>>,
    /// Omit to keep, null to clear, or provide 1 through 10080 minutes.
    #[serde(default, deserialize_with = "nullable_patch")]
    #[schema(minimum = 1, maximum = 10080)]
    pub(crate) scheduled_duration_minutes: Option<Option<u32>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct SessionResponse {
    pub(crate) id: String,
    pub(crate) room_id: String,
    pub(crate) status: SessionStatusDto,
    /// Effective display title, using the current room title when no custom title is set.
    pub(crate) title: String,
    pub(crate) title_overridden: bool,
    #[serde(with = "super::super::rfc3339::option")]
    pub(crate) scheduled_start_at: Option<OffsetDateTime>,
    pub(crate) scheduled_duration_minutes: Option<u32>,
    #[serde(with = "super::super::rfc3339::option")]
    pub(crate) started_at: Option<OffsetDateTime>,
    #[serde(with = "super::super::rfc3339::option")]
    pub(crate) ended_at: Option<OffsetDateTime>,
    /// `None` means the session inherits its room's value.
    #[serde(default)]
    pub(crate) visibility: Option<SessionVisibilityDto>,
    #[serde(default)]
    pub(crate) admission: Option<AdmissionPolicyDto>,
    #[serde(with = "super::super::rfc3339")]
    pub(crate) created_at: OffsetDateTime,
    #[serde(with = "super::super::rfc3339")]
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

impl SessionResponse {
    pub(crate) fn from_session(session: Session, room_title: &str) -> Self {
        Self {
            id: session.id.to_string(),
            room_id: session.room_id.to_string(),
            status: session.status.into(),
            title: session
                .title
                .as_ref()
                .map_or_else(|| room_title.to_owned(), |title| title.as_str().to_owned()),
            title_overridden: session.title.is_some(),
            scheduled_start_at: session.scheduled_start_at,
            scheduled_duration_minutes: session.scheduled_duration_minutes.map(|value| value.get()),
            started_at: session.started_at,
            ended_at: session.ended_at,
            visibility: session.visibility.map(Into::into),
            admission: session.admission.map(Into::into),
            created_at: session.created_at,
            updated_at: session.updated_at,
        }
    }
}

pub(crate) async fn response_with_room(
    state: &crate::http::state::AppState,
    session: Session,
) -> Result<SessionResponse, crate::http::error::ApiError> {
    let room = state
        .rooms
        .get(session.room_id)
        .await
        .map_err(|_| crate::http::error::ApiError::internal("room lookup failed"))?
        .ok_or_else(|| crate::http::error::ApiError::internal("session room missing"))?;
    Ok(SessionResponse::from_session(session, room.title.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mahfel_domain::{RoomId, SessionTitle};
    use serde_json::json;

    #[test]
    fn details_patch_distinguishes_omitted_null_and_value() {
        let omitted: UpdateSessionDetailsRequest = serde_json::from_str("{}").expect("empty patch");
        assert!(omitted.title.is_none());
        let clear: UpdateSessionDetailsRequest = serde_json::from_str(
            r#"{"title":null,"scheduled_start_at":null,"scheduled_duration_minutes":null}"#,
        )
        .expect("clear patch");
        assert_eq!(clear.title, Some(None));
        assert_eq!(clear.scheduled_start_at, Some(None));
        assert_eq!(clear.scheduled_duration_minutes, Some(None));
        let set: UpdateSessionDetailsRequest =
            serde_json::from_str(r#"{"title":"Review","scheduled_duration_minutes":30}"#)
                .expect("set patch");
        assert_eq!(set.title, Some(Some("Review".to_owned())));
        assert_eq!(set.scheduled_duration_minutes, Some(Some(30)));
    }

    #[test]
    fn response_uses_current_room_title_when_unset() {
        let mut session = Session::schedule(RoomId::new(), None, OffsetDateTime::now_utc());
        let inherited = SessionResponse::from_session(session.clone(), "New room title");
        assert_eq!(inherited.title, "New room title");
        assert!(!inherited.title_overridden);
        session.title = Some(SessionTitle::new("Custom".to_owned()).expect("title"));
        let custom = SessionResponse::from_session(session, "New room title");
        assert_eq!(custom.title, "Custom");
        assert!(custom.title_overridden);
    }

    #[test]
    fn session_wire_timestamps_are_rfc3339_strings() {
        let start = OffsetDateTime::parse(
            "2026-09-26T12:30:00+03:30",
            &time::format_description::well_known::Rfc3339,
        )
        .expect("valid start");
        let session = Session::schedule(RoomId::new(), Some(start), OffsetDateTime::now_utc());
        let response = SessionResponse::from_session(session, "Room");
        let value = serde_json::to_value(response).expect("serialize session");

        assert_eq!(value["scheduled_start_at"], "2026-09-26T12:30:00+03:30");
        assert!(value["created_at"].as_str().is_some());
        assert!(value["updated_at"].as_str().is_some());
        assert!(value["started_at"].is_null());

        let request: CreateSessionRequest = serde_json::from_value(json!({
            "scheduled_start_at": "2026-09-26T12:30:00+03:30"
        }))
        .expect("deserialize create request");
        assert_eq!(request.scheduled_start_at, Some(start));

        let patch: UpdateSessionDetailsRequest = serde_json::from_value(json!({
            "scheduled_start_at": "2026-09-26T12:30:00+03:30"
        }))
        .expect("deserialize update request");
        assert_eq!(patch.scheduled_start_at, Some(Some(start)));
    }
}
