use super::dto::RoomResponse;
use crate::http::{
    auth::Authenticated, error::ApiError, extractor::ValidatedPath, state::AppState,
};
use axum::{Json, extract::State};
use mahfel_application::rooms::join_room;
use mahfel_domain::RoomId;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/v1/rooms/{room_id}/join",
    tag = "rooms",
    params(("room_id" = String, Path, description = "Room identifier (UUID)")),
    responses(
        (status = 200, description = "Room joined, or already a member", body = RoomResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Room does not exist or is invite-only"),
        (status = 409, description = "Room is archived"),
    ),
)]
pub(crate) async fn join(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(room_id): ValidatedPath<Uuid>,
) -> Result<Json<RoomResponse>, ApiError> {
    let room = join_room(
        state.rooms.as_ref(),
        RoomId::from_uuid(room_id),
        principal.actor_id,
    )
    .await?;
    Ok(Json(room.into()))
}
