use super::dto::RoomResponse;
use crate::http::{
    auth::Authenticated, error::ApiError, extractor::ValidatedPath, state::AppState,
};
use axum::{Json, extract::State};
use mahfel_application::rooms::get_room as load_room;
use mahfel_domain::RoomId;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/v1/rooms/{room_id}",
    summary = "Get a room",
    tag = "rooms",
    params(("room_id" = String, Path, description = "Room identifier (UUID)")),
    responses(
        (status = 200, description = "Room", body = RoomResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Room does not exist or the caller is not a member"),
    ),
)]
pub(crate) async fn get_room(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(room_id): ValidatedPath<Uuid>,
) -> Result<Json<RoomResponse>, ApiError> {
    let room = load_room(
        state.rooms.as_ref(),
        RoomId::from_uuid(room_id),
        principal.actor_id,
    )
    .await?;
    room.map(RoomResponse::from)
        .map(Json)
        .ok_or_else(|| ApiError::not_found("room not found"))
}
