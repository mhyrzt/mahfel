use super::dto::{
    CollaborationDocumentsResponse, CollaborationEventsResponse, EditGrantRequest, EventsQuery,
    ParticipantGrantResponse, ParticipantGrantsResponse, RegisterImageRequest, ViewportRequest,
};
use crate::http::{auth::Authenticated, error::ApiError, state::AppState};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use mahfel_application::ports::{CollaborationStore, NewCollaborationEvent, StoreError};
use mahfel_domain::{CollaborationEventKind, CollaborationKind, SessionId};
use serde_json::json;
use uuid::Uuid;

fn store(state: &AppState) -> Result<&dyn CollaborationStore, ApiError> {
    state
        .collaboration
        .as_deref()
        .ok_or_else(|| ApiError::unavailable("collaboration unavailable"))
}

fn map_store(error: StoreError) -> ApiError {
    match error {
        StoreError::NotFound(_) | StoreError::MissingReference(_) => {
            ApiError::not_found("collaboration unavailable")
        }
        StoreError::Conflict(_) => ApiError::unprocessable("collaboration state changed"),
        StoreError::Unexpected(_) => ApiError::unavailable("collaboration unavailable"),
    }
}

#[utoipa::path(get, path = "/v1/sessions/{session_id}/collaboration", tag = "collaboration",
    params(("session_id" = String, Path)),
    responses((status = 200, body = CollaborationDocumentsResponse), (status = 404)))]
pub(crate) async fn documents(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    Path(session_id): Path<Uuid>,
) -> Result<Json<CollaborationDocumentsResponse>, ApiError> {
    let store = store(&state)?;
    let session_id = SessionId::from_uuid(session_id);
    let access = store
        .access(session_id, principal.actor_id)
        .await
        .map_err(map_store)?;
    if !access.can_read {
        return Err(ApiError::not_found("collaboration unavailable"));
    }
    let session = state
        .sessions
        .get(session_id)
        .await
        .map_err(map_store)?
        .ok_or_else(|| ApiError::not_found("session unavailable"))?;
    let mut documents = Vec::with_capacity(2);
    for kind in [CollaborationKind::Notes, CollaborationKind::Whiteboard] {
        let document = store
            .document(session_id, kind)
            .await
            .map_err(map_store)?
            .ok_or_else(|| ApiError::not_found("session unavailable"))?;
        documents.push(document.into());
    }
    Ok(Json(CollaborationDocumentsResponse {
        room_id: session.room_id.to_string(),
        documents,
        can_edit: access.can_edit,
        can_manage: access.can_manage,
        live: access.live,
    }))
}

#[utoipa::path(post, path = "/v1/sessions/{session_id}/collaboration/{kind}/images", tag = "collaboration",
    params(("session_id" = String, Path), ("kind" = String, Path)),
    request_body = RegisterImageRequest,
    responses((status = 204), (status = 404), (status = 422)))]
pub(crate) async fn register_image(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    Path((session_id, kind)): Path<(Uuid, String)>,
    Json(request): Json<RegisterImageRequest>,
) -> Result<axum::http::StatusCode, ApiError> {
    let kind = match kind.as_str() {
        "notes" => CollaborationKind::Notes,
        "whiteboard" => CollaborationKind::Whiteboard,
        _ => return Err(ApiError::not_found("collaboration document unavailable")),
    };
    store(&state)?
        .register_image(
            SessionId::from_uuid(session_id),
            kind,
            principal.actor_id,
            mahfel_domain::FileId::from_uuid(request.file_id),
        )
        .await
        .map_err(map_store)?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(get, path = "/v1/sessions/{session_id}/collaboration/events", tag = "collaboration",
    params(("session_id" = String, Path), ("after_sequence" = Option<i64>, Query), ("limit" = Option<u32>, Query)),
    responses((status = 200, body = CollaborationEventsResponse), (status = 404)))]
pub(crate) async fn events(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    Path(session_id): Path<Uuid>,
    Query(query): Query<EventsQuery>,
) -> Result<Json<CollaborationEventsResponse>, ApiError> {
    if query.after_sequence.is_some_and(|sequence| sequence < 0)
        || query.limit.is_some_and(|limit| !(1..=100).contains(&limit))
    {
        return Err(ApiError::bad_request("invalid event cursor or limit"));
    }
    let events = store(&state)?
        .events(
            SessionId::from_uuid(session_id),
            principal.actor_id,
            query.after_sequence.unwrap_or(0),
            query.limit.unwrap_or(100),
        )
        .await
        .map_err(map_store)?;
    Ok(Json(CollaborationEventsResponse {
        events: events.into_iter().map(Into::into).collect(),
    }))
}

#[utoipa::path(put, path = "/v1/sessions/{session_id}/collaboration/grants/{actor_id}", tag = "collaboration",
    params(("session_id" = String, Path), ("actor_id" = String, Path)),
    request_body = EditGrantRequest,
    responses((status = 204), (status = 404)))]
pub(crate) async fn set_grant(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    Path((session_id, actor_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<EditGrantRequest>,
) -> Result<axum::http::StatusCode, ApiError> {
    store(&state)?
        .set_edit_grant(
            SessionId::from_uuid(session_id),
            principal.actor_id,
            mahfel_domain::ActorId::from_uuid(actor_id),
            request.can_edit,
            Uuid::new_v4(),
        )
        .await
        .map_err(map_store)?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(get, path = "/v1/sessions/{session_id}/collaboration/grants", tag = "collaboration",
    params(("session_id" = String, Path)),
    responses((status = 200, body = ParticipantGrantsResponse), (status = 404)))]
pub(crate) async fn grants(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    Path(session_id): Path<Uuid>,
) -> Result<Json<ParticipantGrantsResponse>, ApiError> {
    let participants = store(&state)?
        .participants(SessionId::from_uuid(session_id), principal.actor_id)
        .await
        .map_err(map_store)?;
    Ok(Json(ParticipantGrantsResponse {
        participants: participants
            .into_iter()
            .map(|participant| ParticipantGrantResponse {
                actor_id: participant.actor_id.to_string(),
                display_name: participant.display_name,
                can_edit: participant.can_edit,
            })
            .collect(),
    }))
}

#[utoipa::path(post, path = "/v1/sessions/{session_id}/collaboration/viewport", tag = "collaboration",
    params(("session_id" = String, Path)), request_body = ViewportRequest,
    responses((status = 204), (status = 404)))]
pub(crate) async fn update_viewport(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    Path(session_id): Path<Uuid>,
    Json(request): Json<ViewportRequest>,
) -> Result<axum::http::StatusCode, ApiError> {
    if !matches!(request.surface.as_str(), "notes" | "whiteboard")
        || !request.x.is_finite()
        || !request.y.is_finite()
        || request.x.abs() > 10_000_000.0
        || request.y.abs() > 10_000_000.0
        || request
            .zoom
            .is_some_and(|zoom| !zoom.is_finite() || !(0.1..=5.0).contains(&zoom))
    {
        return Err(ApiError::bad_request("invalid viewport"));
    }
    store(&state)?.append(&NewCollaborationEvent {
        session_id: SessionId::from_uuid(session_id), id: Uuid::new_v4(),
        actor_id: principal.actor_id, document_id: None,
        kind: CollaborationEventKind::Viewport, update: None,
        data: json!({"surface": request.surface, "x": request.x, "y": request.y, "zoom": request.zoom}),
    }).await.map_err(map_store)?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
