use super::dto::RoomResponse;
use crate::http::{
    auth::Authenticated,
    error::ApiError,
    extractor::ValidatedPath,
    state::{AppState, RoomStoreState},
};
use axum::{Json, extract::State};
use mahfel_application::rooms::get_room as load_room;
use mahfel_domain::RoomId;
use uuid::Uuid;

pub(super) async fn get_room<R: RoomStoreState>(
    State(state): State<AppState<R>>,
    Authenticated(principal): Authenticated,
    ValidatedPath(room_id): ValidatedPath<Uuid>,
) -> Result<Json<RoomResponse>, ApiError> {
    let room = load_room(&state.rooms, RoomId::from_uuid(room_id), principal.user_id).await?;
    room.map(RoomResponse::from)
        .map(Json)
        .ok_or_else(|| ApiError::not_found("room not found"))
}
