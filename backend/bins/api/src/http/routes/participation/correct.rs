use super::dto::{CorrectIntervalRequest, IntervalResponse};
use crate::http::{
    auth::Authenticated,
    error::ApiError,
    extractor::{ValidatedJson, ValidatedPath},
    state::AppState,
};
use axum::{Json, extract::State};
use mahfel_application::participation::{CorrectInterval, correct_interval};
use mahfel_domain::IntervalId;
use time::OffsetDateTime;
use uuid::Uuid;

/// Manager-only: replace an interval with a corrected one.
///
/// The original row is retained as superseded, so the original measurement and
/// the reason it changed both survive.
#[utoipa::path(
    post,
    path = "/v1/participation/intervals/{interval_id}/correct",
    tag = "participation",
    params(("interval_id" = String, Path, description = "Interval identifier (UUID)")),
    request_body = CorrectIntervalRequest,
    responses(
        (status = 200, description = "Replacement interval", body = IntervalResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Caller is not a manager"),
        (status = 404, description = "Interval not found"),
        (status = 409, description = "Interval already superseded or the correction is invalid"),
    ),
)]
pub(crate) async fn correct(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(interval_id): ValidatedPath<Uuid>,
    ValidatedJson(body): ValidatedJson<CorrectIntervalRequest>,
) -> Result<Json<IntervalResponse>, ApiError> {
    let replacement = correct_interval(
        state.participation.as_ref(),
        state.sessions.as_ref(),
        state.rooms.as_ref(),
        CorrectInterval {
            actor: principal.actor_id,
            interval_id: IntervalId::from_uuid(interval_id),
            reason: body.reason.into(),
            started_at: body.started_at,
            ended_at: body.ended_at,
        },
    )
    .await?;
    Ok(Json(IntervalResponse::from_interval(
        &replacement,
        OffsetDateTime::now_utc(),
    )))
}
