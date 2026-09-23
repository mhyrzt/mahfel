use crate::http::{
    auth::Authenticated,
    error::ApiError,
    extractor::{ValidatedJson, ValidatedPath},
    state::AppState,
};
use axum::{Json, extract::State};
use mahfel_application::chat::ChatOperationError;
use mahfel_domain::RoomId;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct AdvanceReadMarkerRequest {
    sequence: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ReadMarkerResponse {
    sequence: i64,
}

#[utoipa::path(
    get,
    path = "/v1/rooms/{room_id}/read-marker",
    summary = "Get only the caller's room read position",
    operation_id = "getRoomReadMarker",
    tag = "chat",
    params(("room_id" = Uuid, Path)),
    responses((status = 200, body = ReadMarkerResponse), (status = 404)),
)]
pub(crate) async fn get_marker(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(room_id): ValidatedPath<Uuid>,
) -> Result<Json<ReadMarkerResponse>, ApiError> {
    let sequence = state
        .chat
        .read_marker(RoomId::from_uuid(room_id), principal.actor_id)
        .await
        .map_err(ChatOperationError::from)?;
    Ok(Json(ReadMarkerResponse { sequence }))
}

#[utoipa::path(
    put,
    path = "/v1/rooms/{room_id}/read-marker",
    summary = "Move only the caller's room read position forward",
    operation_id = "advanceRoomReadMarker",
    tag = "chat",
    params(("room_id" = Uuid, Path)),
    request_body = AdvanceReadMarkerRequest,
    responses((status = 200, body = ReadMarkerResponse), (status = 400), (status = 404), (status = 409)),
)]
pub(crate) async fn advance_marker(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(room_id): ValidatedPath<Uuid>,
    ValidatedJson(body): ValidatedJson<AdvanceReadMarkerRequest>,
) -> Result<Json<ReadMarkerResponse>, ApiError> {
    if body.sequence < 0 {
        return Err(ApiError::bad_request("invalid read sequence"));
    }
    let sequence = state
        .chat
        .advance_read_marker(
            RoomId::from_uuid(room_id),
            principal.actor_id,
            body.sequence,
            OffsetDateTime::now_utc(),
        )
        .await
        .map_err(ChatOperationError::from)?;
    Ok(Json(ReadMarkerResponse { sequence }))
}
