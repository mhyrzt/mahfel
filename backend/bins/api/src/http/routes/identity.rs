use crate::http::{
    auth::{self, Authenticated},
    error::ApiError,
    state::{AppState, RoomStoreState},
};
use axum::{Json, Router, extract::State, routing::get};
use mahfel_application::ports::Action;
use serde::Serialize;

#[derive(Serialize)]
struct IdentityResponse {
    user_id: String,
}

pub(super) fn route<R: RoomStoreState>() -> Router<AppState<R>> {
    Router::new().route("/me", get(me::<R>))
}

async fn me<R: RoomStoreState>(
    State(state): State<AppState<R>>,
    Authenticated(principal): Authenticated,
) -> Result<Json<IdentityResponse>, ApiError> {
    auth::authorize(&state, &principal, Action::ViewOwnIdentity)?;
    Ok(Json(IdentityResponse {
        user_id: principal.user_id.to_string(),
    }))
}
