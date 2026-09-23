use super::dto::{SessionResponse, UpdateSessionDetailsRequest, response_with_room};
use crate::http::{
    auth::Authenticated,
    error::ApiError,
    extractor::{ValidatedJson, ValidatedPath},
    state::AppState,
};
use axum::{Json, extract::State};
use mahfel_application::sessions::{UpdateSessionDetails, update_session_details};
use mahfel_domain::SessionId;
use uuid::Uuid;

#[utoipa::path(
    patch,
    operation_id = "updateSessionDetails",
    path = "/v1/sessions/{session_id}",
    summary = "Update session title and schedule",
    tag = "sessions",
    params(("session_id" = String, Path, description = "Session identifier (UUID)")),
    request_body = UpdateSessionDetailsRequest,
    responses(
        (status = 200, description = "Session details updated", body = SessionResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Caller lacks manage_session"),
        (status = 404, description = "Session or room is not visible"),
        (status = 422, description = "Invalid title or advisory duration"),
    ),
)]
pub(crate) async fn update(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(session_id): ValidatedPath<Uuid>,
    ValidatedJson(body): ValidatedJson<UpdateSessionDetailsRequest>,
) -> Result<Json<SessionResponse>, ApiError> {
    let session = update_session_details(
        state.sessions.as_ref(),
        state.rooms.as_ref(),
        UpdateSessionDetails {
            principal,
            session_id: SessionId::from_uuid(session_id),
            title: body.title,
            scheduled_start_at: body.scheduled_start_at,
            scheduled_duration_minutes: body.scheduled_duration_minutes,
        },
    )
    .await?;
    Ok(Json(response_with_room(&state, session).await?))
}
