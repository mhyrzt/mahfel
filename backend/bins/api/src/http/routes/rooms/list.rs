use super::dto::RoomResponse;
use crate::http::{
    auth::Authenticated,
    error::ApiError,
    extractor::ValidatedPagination,
    pagination::Paginated,
    state::{AppState, RoomStoreState},
};
use axum::{Json, extract::State};
use mahfel_application::rooms::list_joined;

pub(super) async fn list_rooms<R: RoomStoreState>(
    State(state): State<AppState<R>>,
    Authenticated(principal): Authenticated,
    ValidatedPagination(pagination): ValidatedPagination,
) -> Result<Json<Paginated<RoomResponse>>, ApiError> {
    let rooms = list_joined(
        &state.rooms,
        principal.user_id,
        pagination.limit(),
        pagination.offset(),
    )
    .await?;
    let data = rooms.into_iter().map(RoomResponse::from).collect();
    Ok(Json(Paginated::new(data, pagination, None)))
}
