pub(crate) mod service_principals;

use crate::http::{
    auth::{self, Authenticated},
    error::ApiError,
    state::AppState,
};
use axum::{Json, Router, extract::State, routing::get};
use mahfel_application::ports::Capability;
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
pub(crate) async fn ping(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
) -> Result<Json<AdminResponse>, ApiError> {
    auth::authorize_global(&state, &principal, Capability::ManageSystem)?;
    Ok(Json(AdminResponse { ok: true }))
}

pub(crate) fn route() -> Router<AppState> {
    Router::new()
        .route("/admin/ping", get(ping))
        .merge(service_principals::route())
}
