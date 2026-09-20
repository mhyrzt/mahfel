use crate::http::{
    auth::{self, Authenticated},
    error::ApiError,
    state::{AppState, RoomStoreState},
};
use axum::{Json, Router, extract::State, routing::get};
use mahfel_application::ports::Action;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub(crate) struct AdminResponse {
    ok: bool,
}

#[utoipa::path(
    get,
    path = "/v1/admin/ping",
    tag = "admin",
    responses(
        (status = 200, description = "Admin access granted", body = AdminResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Permission denied"),
    ),
)]
pub(crate) async fn ping<R: RoomStoreState>(
    State(state): State<AppState<R>>,
    Authenticated(principal): Authenticated,
) -> Result<Json<AdminResponse>, ApiError> {
    auth::authorize(&state, &principal, Action::ManageSystem)?;
    Ok(Json(AdminResponse { ok: true }))
}

pub(crate) fn route<R: RoomStoreState>() -> Router<AppState<R>> {
    Router::new().route("/admin/ping", get(ping::<R>))
}
