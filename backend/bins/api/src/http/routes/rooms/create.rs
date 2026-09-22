use super::dto::{CreateRoomRequest, RoomResponse, RoomVisibilityDto};
use crate::http::{
    auth::{self, Authenticated},
    error::ApiError,
    extractor::ValidatedJson,
    state::AppState,
};
use axum::{Json, extract::State, http::StatusCode};
use mahfel_application::{
    ports::Capability,
    rooms::{CreateRoom, create_room},
};

#[utoipa::path(
    post,
    path = "/v1/rooms",
    operation_id = "createRoom",
    tag = "rooms",
    request_body = CreateRoomRequest,
    responses(
        (status = 201, description = "Room created", body = RoomResponse),
        (status = 401, description = "Authentication required"),
        (status = 422, description = "Invalid room title"),
    ),
)]
pub(crate) async fn create(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedJson(body): ValidatedJson<CreateRoomRequest>,
) -> Result<(StatusCode, Json<RoomResponse>), ApiError> {
    auth::authorize_global(&state, &principal, Capability::CreateRoom)?;
    let room = create_room(
        state.rooms.as_ref(),
        CreateRoom {
            title: body.title,
            visibility: body
                .visibility
                .unwrap_or(RoomVisibilityDto::InviteOnly)
                .into(),
            created_by: principal.actor_id,
        },
    )
    .await?;
    Ok((StatusCode::CREATED, Json(room.into())))
}
