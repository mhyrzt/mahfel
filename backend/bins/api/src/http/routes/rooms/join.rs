use super::dto::RoomResponse;
use crate::http::{
    auth::Authenticated,
    error::ApiError,
    extractor::ValidatedPath,
    state::{AppState, RoomStoreState},
};
use axum::{Json, extract::State};
use mahfel_application::rooms::join_room;
use mahfel_domain::RoomId;
use uuid::Uuid;

pub(super) async fn join<R: RoomStoreState>(
    State(state): State<AppState<R>>,
    Authenticated(principal): Authenticated,
    ValidatedPath(room_id): ValidatedPath<Uuid>,
) -> Result<Json<RoomResponse>, ApiError> {
    let room = join_room(&state.rooms, RoomId::from_uuid(room_id), principal.user_id).await?;
    Ok(Json(room.into()))
}
