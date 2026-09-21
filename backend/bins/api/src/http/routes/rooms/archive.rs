use super::dto::RoomResponse;
use crate::http::{
    auth::Authenticated, error::ApiError, extractor::ValidatedPath, state::AppState,
};
use axum::{Json, extract::State};
use mahfel_application::rooms::archive_room;
use mahfel_domain::RoomId;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/v1/rooms/{room_id}/archive",
    tag = "rooms",
    params(("room_id" = String, Path, description = "Room identifier (UUID)")),
    responses(
        (status = 200, description = "Room archived", body = RoomResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Caller is not a manager"),
        (status = 404, description = "Room does not exist or the caller is not a member"),
        (status = 409, description = "Room is already archived"),
    ),
)]
pub(crate) async fn archive(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(room_id): ValidatedPath<Uuid>,
) -> Result<Json<RoomResponse>, ApiError> {
    let room = archive_room(
        state.rooms.as_ref(),
        RoomId::from_uuid(room_id),
        principal.actor_id,
    )
    .await?;
    Ok(Json(room.into()))
}
