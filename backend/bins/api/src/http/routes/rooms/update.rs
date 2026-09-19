use super::dto::{RoomResponse, UpdateRoomRequest};
use crate::http::{
    auth::Authenticated,
    error::ApiError,
    extractor::{ValidatedJson, ValidatedPath},
    state::{AppState, RoomStoreState},
};
use axum::{Json, extract::State};
use mahfel_application::rooms::{UpdateRoom, update_settings};
use mahfel_domain::RoomId;
use uuid::Uuid;

pub(super) async fn update<R: RoomStoreState>(
    State(state): State<AppState<R>>,
    Authenticated(principal): Authenticated,
    ValidatedPath(room_id): ValidatedPath<Uuid>,
    ValidatedJson(body): ValidatedJson<UpdateRoomRequest>,
) -> Result<Json<RoomResponse>, ApiError> {
    let room = update_settings(
        &state.rooms,
        RoomId::from_uuid(room_id),
        principal.user_id,
        UpdateRoom {
            title: body.title,
            visibility: body.visibility,
        },
    )
    .await?;
    Ok(Json(room.into()))
}
