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

#[utoipa::path(
    get,
    path = "/v1/rooms",
    tag = "rooms",
    params(
        ("page" = Option<u64>, Query, description = "Page number, 1-based"),
        ("per_page" = Option<u64>, Query, description = "Items per page, 1-100"),
    ),
    responses(
        (status = 200, description = "Rooms the caller has joined", body = Paginated<RoomResponse>),
        (status = 401, description = "Authentication required"),
    ),
)]
pub(crate) async fn list_rooms<R: RoomStoreState>(
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
