use super::dto::{CreateRoomRequest, RoomResponse};
use crate::http::{
    auth::{self, Authenticated},
    error::ApiError,
    extractor::ValidatedJson,
    state::{AppState, RoomStoreState},
};
use axum::{Json, extract::State, http::StatusCode};
use mahfel_application::{
    ports::Action,
    rooms::{CreateRoom, create_room},
};
use mahfel_domain::RoomVisibility;

pub(super) async fn create<R: RoomStoreState>(
    State(state): State<AppState<R>>,
    Authenticated(principal): Authenticated,
    ValidatedJson(body): ValidatedJson<CreateRoomRequest>,
) -> Result<(StatusCode, Json<RoomResponse>), ApiError> {
    auth::authorize(&state, &principal, Action::CreateRoom)?;
    let room = create_room(
        &state.rooms,
        CreateRoom {
            title: body.title,
            visibility: body.visibility.unwrap_or(RoomVisibility::InviteOnly),
            created_by: principal.user_id,
        },
    )
    .await?;
    Ok((StatusCode::CREATED, Json(room.into())))
}
