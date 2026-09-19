use super::super::{
    error::ApiError,
    extractor::{ValidatedJson, ValidatedPagination},
    pagination::Paginated,
};
use axum::{Json, Router, routing::get};
use serde::{Deserialize, Serialize};

/// Placeholder payload used by contract tests until real resource routes land.
#[derive(Clone, Serialize, Deserialize)]
pub struct EchoPayload {
    pub message: String,
}

const PLACEHOLDERS: [&str; 4] = ["alpha", "beta", "gamma", "delta"];
const MAX_MESSAGE_LEN: usize = 300;

pub(super) fn route() -> Router {
    Router::new().route("/echo", get(list).post(create))
}

async fn list(
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

async fn create(
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
