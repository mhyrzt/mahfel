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
pub(crate) struct IdentityResponse {
    user_id: String,
}

#[utoipa::path(
    get,
    path = "/v1/me",
    tag = "identity",
    responses(
        (status = 200, description = "Authenticated principal", body = IdentityResponse),
        (status = 401, description = "Authentication required"),
    ),
)]
pub(crate) async fn me<R: RoomStoreState>(
    State(state): State<AppState<R>>,
    Authenticated(principal): Authenticated,
) -> Result<Json<IdentityResponse>, ApiError> {
    auth::authorize(&state, &principal, Action::ViewOwnIdentity)?;
    Ok(Json(IdentityResponse {
        user_id: principal.user_id.to_string(),
    }))
}

pub(crate) fn route<R: RoomStoreState>() -> Router<AppState<R>> {
    Router::new().route("/me", get(me::<R>))
}
