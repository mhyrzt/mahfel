use crate::http::{
    error::ApiError,
    extractor::{ValidatedJson, ValidatedPagination},
    pagination::Paginated,
    state::AppState,
};
use axum::{Json, Router, routing::get};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Placeholder payload used by contract tests until real resource routes land.
#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct EchoPayload {
    pub message: String,
}

const PLACEHOLDERS: [&str; 4] = ["alpha", "beta", "gamma", "delta"];
const MAX_MESSAGE_LEN: usize = 300;

pub(crate) fn route() -> Router<AppState> {
    Router::new().route("/echo", get(list).post(create))
}

#[utoipa::path(
    get,
    path = "/v1/echo",
    tag = "echo",
    params(
        ("page" = Option<u64>, Query, description = "Page number, 1-based"),
        ("per_page" = Option<u64>, Query, description = "Items per page, 1-100"),
    ),
    responses((status = 200, description = "Placeholder page", body = Paginated<EchoPayload>)),
)]
pub(crate) async fn list(
    ValidatedPagination(pagination): ValidatedPagination,
) -> Json<Paginated<EchoPayload>> {
    let total = PLACEHOLDERS.len() as u64;
    let start = (pagination.offset() as usize).min(PLACEHOLDERS.len());
    let end = (start + pagination.limit() as usize).min(PLACEHOLDERS.len());
    let data = PLACEHOLDERS[start..end]
        .iter()
        .map(|message| EchoPayload {
            message: (*message).to_owned(),
        })
        .collect();
    Json(Paginated::new(data, pagination, Some(total)))
}

#[utoipa::path(
    post,
    path = "/v1/echo",
    tag = "echo",
    request_body = EchoPayload,
    responses(
        (status = 200, description = "Echoed payload", body = EchoPayload),
        (status = 422, description = "Validation failed"),
    ),
)]
pub(crate) async fn create(
    ValidatedJson(payload): ValidatedJson<EchoPayload>,
) -> Result<Json<EchoPayload>, ApiError> {
    if payload.message.trim().is_empty() {
        return Err(ApiError::unprocessable("message must not be empty"));
    }
    if payload.message.chars().count() > MAX_MESSAGE_LEN {
        return Err(ApiError::unprocessable(format!(
            "message exceeds {MAX_MESSAGE_LEN} characters"
        )));
    }
    Ok(Json(payload))
}
