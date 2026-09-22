//! Session access overrides and the lobby.

use crate::http::{
    auth::Authenticated, error::ApiError, extractor::ValidatedPath, state::AppState,
};
use axum::{Json, extract::State};
use mahfel_application::admission::{
    AdmissionDecision, Lobby, SessionAccess, SessionAccessView, admit, reject, set_session_access,
    view_lobby, view_session_access,
};
use mahfel_domain::{ActorId, SessionId};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

use super::dto::{AdmissionPolicyDto, SessionVisibilityDto};

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct SetSessionAccessRequest {
    /// Omit to clear an override and inherit the room's value again.
    #[serde(default)]
    pub(crate) visibility: Option<SessionVisibilityDto>,
    #[serde(default)]
    pub(crate) admission: Option<AdmissionPolicyDto>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct SessionAccessResponse {
    pub(crate) session_id: String,
    /// The value that governs this session, after inheritance.
    pub(crate) visibility: SessionVisibilityDto,
    pub(crate) admission: AdmissionPolicyDto,
    /// Whether each axis is overridden rather than inherited from the room.
    pub(crate) visibility_overridden: bool,
    pub(crate) admission_overridden: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct LobbyEntryResponse {
    pub(crate) request_id: String,
    pub(crate) actor_id: String,
    pub(crate) origin: String,
    #[serde(with = "super::super::rfc3339")]
    pub(crate) requested_at: OffsetDateTime,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct LobbyResponse {
    pub(crate) session_id: String,
    pub(crate) entries: Vec<LobbyEntryResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct AdmittedResponse {
    pub(crate) session_id: String,
    pub(crate) actor_id: String,
    pub(crate) participant_id: String,
}

/// Change a session's access overrides.
///
/// Manager-only. An override may only tighten access, so a session cannot expose
/// a room the room itself protects. Omitting a field clears that override.
#[utoipa::path(
    put,
    path = "/v1/sessions/{session_id}/access",
    tag = "admission",
    params(("session_id" = String, Path, description = "Session identifier (UUID)")),
    request_body = SetSessionAccessRequest,
    responses(
        (status = 200, description = "Session access updated", body = SessionAccessResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Caller does not manage the session's room"),
        (status = 404, description = "Session does not exist or is not visible to the caller"),
        (status = 422, description = "An override would loosen the room's settings"),
    ),
)]
pub(crate) async fn set_access(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(session_id): ValidatedPath<Uuid>,
    Json(request): Json<SetSessionAccessRequest>,
) -> Result<Json<SessionAccessResponse>, ApiError> {
    let session_id = SessionId::from_uuid(session_id);
    let view = set_session_access(
        state.sessions.as_ref(),
        state.rooms.as_ref(),
        SessionAccess {
            actor: principal.actor_id,
            session_id,
            visibility: request.visibility.map(Into::into),
            admission: request.admission.map(Into::into),
        },
        OffsetDateTime::now_utc(),
    )
    .await?;
    Ok(Json(response(session_id, view)))
}

/// Read a session's effective access settings.
#[utoipa::path(
    get,
    path = "/v1/sessions/{session_id}/access",
    tag = "admission",
    params(("session_id" = String, Path, description = "Session identifier (UUID)")),
    responses(
        (status = 200, description = "Effective session access", body = SessionAccessResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Caller does not manage the session's room"),
        (status = 404, description = "Session does not exist or is not visible to the caller"),
    ),
)]
pub(crate) async fn get_access(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(session_id): ValidatedPath<Uuid>,
) -> Result<Json<SessionAccessResponse>, ApiError> {
    let session_id = SessionId::from_uuid(session_id);
    let view = view_session_access(
        state.sessions.as_ref(),
        state.rooms.as_ref(),
        principal.actor_id,
        session_id,
    )
    .await?;
    Ok(Json(response(session_id, view)))
}

/// Manager-only: who is waiting to join this session.
#[utoipa::path(
    get,
    path = "/v1/sessions/{session_id}/lobby",
    tag = "admission",
    params(("session_id" = String, Path, description = "Session identifier (UUID)")),
    responses(
        (status = 200, description = "Waiting principals", body = LobbyResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Caller does not manage the session's room"),
        (status = 404, description = "Session does not exist or is not visible to the caller"),
    ),
)]
pub(crate) async fn get_lobby(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(session_id): ValidatedPath<Uuid>,
) -> Result<Json<LobbyResponse>, ApiError> {
    let session_id = SessionId::from_uuid(session_id);
    let lobby: Lobby = view_lobby(
        state.admissions.as_ref(),
        state.sessions.as_ref(),
        state.rooms.as_ref(),
        principal.actor_id,
        session_id,
        None,
    )
    .await?;
    Ok(Json(LobbyResponse {
        session_id: session_id.to_string(),
        entries: lobby
            .entries
            .into_iter()
            .map(|entry| LobbyEntryResponse {
                request_id: entry.request_id.to_string(),
                actor_id: entry.actor_id.to_string(),
                origin: entry.origin.as_str().to_owned(),
                requested_at: entry.requested_at,
            })
            .collect(),
    }))
}

/// Admit a waiting principal. Requires `admit_participant`.
#[utoipa::path(
    post,
    path = "/v1/sessions/{session_id}/lobby/{actor_id}/admit",
    tag = "admission",
    params(
        ("session_id" = String, Path, description = "Session identifier (UUID)"),
        ("actor_id" = String, Path, description = "Waiting principal's actor id (UUID)"),
    ),
    responses(
        (status = 200, description = "Principal admitted", body = AdmittedResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Caller does not manage the session's room"),
        (status = 404, description = "Session does not exist, or the principal is not waiting"),
        (status = 409, description = "The request was already decided"),
    ),
)]
pub(crate) async fn admit_principal(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath((session_id, actor_id)): ValidatedPath<(Uuid, Uuid)>,
) -> Result<Json<AdmittedResponse>, ApiError> {
    let session_id = SessionId::from_uuid(session_id);
    let target = ActorId::from_uuid(actor_id);
    let participant = admit(
        state.realtime.stores.participants.as_ref(),
        state.participation.as_ref(),
        state.admissions.as_ref(),
        state.sessions.as_ref(),
        state.rooms.as_ref(),
        AdmissionDecision {
            actor: principal.actor_id,
            session_id,
            target,
        },
        OffsetDateTime::now_utc(),
    )
    .await?;
    Ok(Json(AdmittedResponse {
        session_id: session_id.to_string(),
        actor_id: target.to_string(),
        participant_id: participant.id.to_string(),
    }))
}

/// Reject a waiting principal. Terminates the waiting state and grants nothing.
#[utoipa::path(
    post,
    path = "/v1/sessions/{session_id}/lobby/{actor_id}/reject",
    tag = "admission",
    params(
        ("session_id" = String, Path, description = "Session identifier (UUID)"),
        ("actor_id" = String, Path, description = "Waiting principal's actor id (UUID)"),
    ),
    responses(
        (status = 204, description = "Principal rejected"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Caller does not manage the session's room"),
        (status = 404, description = "Session does not exist, or the principal is not waiting"),
        (status = 409, description = "The request was already decided"),
    ),
)]
pub(crate) async fn reject_principal(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath((session_id, actor_id)): ValidatedPath<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, ApiError> {
    reject(
        state.admissions.as_ref(),
        state.sessions.as_ref(),
        state.rooms.as_ref(),
        AdmissionDecision {
            actor: principal.actor_id,
            session_id: SessionId::from_uuid(session_id),
            target: ActorId::from_uuid(actor_id),
        },
        OffsetDateTime::now_utc(),
    )
    .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

fn response(session_id: SessionId, view: SessionAccessView) -> SessionAccessResponse {
    SessionAccessResponse {
        session_id: session_id.to_string(),
        visibility: view.visibility.into(),
        admission: view.admission.into(),
        visibility_overridden: view.visibility_overridden,
        admission_overridden: view.admission_overridden,
    }
}
