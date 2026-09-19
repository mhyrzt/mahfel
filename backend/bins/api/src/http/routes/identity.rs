use super::super::{
    auth::{self, Authenticated},
    error::ApiError,
    state::AppState,
};
use axum::{Json, Router, extract::State, routing::get};
use mahfel_application::ports::Action;
use serde::Serialize;

#[derive(Serialize)]
struct IdentityResponse {
    user_id: String,
}

pub(super) fn route() -> Router<AppState> {
    Router::new().route("/me", get(me))
}

async fn me(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
) -> Result<Json<IdentityResponse>, ApiError> {
    auth::authorize(&state, &principal, Action::ViewOwnIdentity)?;
    Ok(Json(IdentityResponse {
        user_id: principal.user_id.to_string(),
    }))
}
