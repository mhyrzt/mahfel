use super::dto::{RoomResponse, UpdateRoomRequest};
use crate::http::{
    auth::Authenticated,
    error::ApiError,
    extractor::{ValidatedJson, ValidatedPath},
    state::{AppState, RoomStoreState},
};
use axum::{Json, extract::State};
use mahfel_application::rooms::{UpdateRoom, update_settings};
use mahfel_domain::{RoomId, RoomVisibility};
use uuid::Uuid;

#[utoipa::path(
    patch,
    path = "/v1/rooms/{room_id}",
    tag = "rooms",
    params(("room_id" = String, Path, description = "Room identifier (UUID)")),
    request_body = UpdateRoomRequest,
    responses(
        (status = 200, description = "Room updated", body = RoomResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Caller is not a manager"),
        (status = 404, description = "Room does not exist or the caller is not a member"),
        (status = 409, description = "Room is archived"),
        (status = 422, description = "Invalid room title"),
    ),
)]
pub(crate) async fn update<R: RoomStoreState>(
    State(state): State<AppState<R>>,
    Authenticated(principal): Authenticated,
    ValidatedPath(room_id): ValidatedPath<Uuid>,
    ValidatedJson(body): ValidatedJson<UpdateRoomRequest>,
) -> Result<Json<RoomResponse>, ApiError> {
    let room = update_settings(
        &state.rooms,
        RoomId::from_uuid(room_id),
        principal.actor_id,
        UpdateRoom {
            title: body.title,
            visibility: body.visibility.map(RoomVisibility::from),
        },
    )
    .await?;
    Ok(Json(room.into()))
}
