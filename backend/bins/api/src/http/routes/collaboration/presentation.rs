use axum::{
    Json,
    extract::{Path, State},
};
use mahfel_application::{
    collaboration::{PresentationAction, PresentationError, change_presentation},
    ports::StoreError,
};
use mahfel_domain::{FileId, SessionId};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::http::{auth::Authenticated, error::ApiError, state::AppState};

use super::dto::{EventResponse, PresentationRequest};

#[derive(Serialize, ToSchema)]
pub(crate) struct PresentationStateResponse {
    file_ids: Vec<Uuid>,
    current_page: u32,
    total_pages: u32,
}

#[utoipa::path(get, path = "/v1/sessions/{session_id}/collaboration/presentation", tag = "collaboration",
    params(("session_id" = String, Path)), responses((status = 200, body = Option<PresentationStateResponse>), (status = 404)))]
pub(crate) async fn get_presentation(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    Path(session_id): Path<Uuid>,
) -> Result<Json<Option<PresentationStateResponse>>, ApiError> {
    let store = state
        .collaboration
        .as_deref()
        .ok_or_else(|| ApiError::unavailable("collaboration unavailable"))?;
    let current = store
        .presentation(SessionId::from_uuid(session_id), principal.actor_id)
        .await
        .map_err(|_| ApiError::not_found("presentation unavailable"))?;
    Ok(Json(current.map(|state| PresentationStateResponse {
        file_ids: state.file_ids.into_iter().map(FileId::as_uuid).collect(),
        current_page: state.current_page,
        total_pages: state.total_pages,
    })))
}

#[utoipa::path(post, path = "/v1/sessions/{session_id}/collaboration/presentation", tag = "collaboration",
    params(("session_id" = String, Path)), request_body = PresentationRequest,
    responses((status = 200, body = EventResponse), (status = 400), (status = 404)))]
pub(crate) async fn update_presentation(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    Path(session_id): Path<Uuid>,
    Json(request): Json<PresentationRequest>,
) -> Result<Json<EventResponse>, ApiError> {
    let store = state
        .collaboration
        .as_deref()
        .ok_or_else(|| ApiError::unavailable("collaboration unavailable"))?;
    let action = match request {
        PresentationRequest::Select { file_ids } => {
            PresentationAction::Select(file_ids.into_iter().map(FileId::from_uuid).collect())
        }
        PresentationRequest::Page { page } => PresentationAction::Page(page),
        PresentationRequest::Stop => PresentationAction::Stop,
    };
    let event = change_presentation(
        store,
        state.files.as_ref(),
        state.storage.as_ref(),
        state.sessions.as_ref(),
        SessionId::from_uuid(session_id),
        principal.actor_id,
        action,
    )
    .await
    .map_err(|error| match error {
        PresentationError::Denied | PresentationError::Store(StoreError::NotFound(_)) => {
            ApiError::not_found("presentation unavailable")
        }
        PresentationError::Invalid | PresentationError::Store(StoreError::Conflict(_)) => {
            ApiError::bad_request("invalid presentation change")
        }
        _ => ApiError::unavailable("presentation unavailable"),
    })?;
    Ok(Json(event.into()))
}
