use crate::http::{
    auth::Authenticated, error::ApiError, extractor::ValidatedPath, state::AppState,
};
use axum::{Json, extract::State};
use mahfel_application::invite::{InviteLinkQuery, list_links};
use time::OffsetDateTime;
use uuid::Uuid;

use super::dto::{InviteLinkListResponse, InviteLinkResponse};

#[derive(Debug, serde::Deserialize)]
pub(crate) struct LimitQuery {
    pub(crate) limit: Option<u32>,
}

/// List a room's invite links, newest first.
///
/// Manager-only. Revoked and expired links are included so a manager sees the
/// full history, not only what is currently live.
#[utoipa::path(
    get,
    path = "/v1/rooms/{room_id}/invite-links",
    tag = "invite",
    params(
        ("room_id" = String, Path, description = "Room identifier (UUID)"),
        ("limit" = Option<u32>, Query, description = "Page size, 1-200"),
    ),
    responses(
        (status = 200, description = "Invite links for the room", body = InviteLinkListResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Caller does not manage the room"),
        (status = 404, description = "Room does not exist or is not visible to the caller"),
    ),
)]
pub(crate) async fn list_for_room(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(room_id): ValidatedPath<Uuid>,
    axum::extract::Query(query): axum::extract::Query<LimitQuery>,
) -> Result<Json<InviteLinkListResponse>, ApiError> {
    list(
        &state,
        principal,
        super::dto::room_target(room_id),
        query.limit,
    )
    .await
}

/// List a session's invite links, newest first.
#[utoipa::path(
    get,
    path = "/v1/sessions/{session_id}/invite-links",
    tag = "invite",
    params(
        ("session_id" = String, Path, description = "Session identifier (UUID)"),
        ("limit" = Option<u32>, Query, description = "Page size, 1-200"),
    ),
    responses(
        (status = 200, description = "Invite links for the session", body = InviteLinkListResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Caller does not manage the session's room"),
        (status = 404, description = "Session does not exist or is not visible to the caller"),
    ),
)]
pub(crate) async fn list_for_session(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(session_id): ValidatedPath<Uuid>,
    axum::extract::Query(query): axum::extract::Query<LimitQuery>,
) -> Result<Json<InviteLinkListResponse>, ApiError> {
    list(
        &state,
        principal,
        super::dto::session_target(session_id),
        query.limit,
    )
    .await
}

async fn list(
    state: &AppState,
    principal: mahfel_application::ports::Principal,
    target: mahfel_application::invite::InviteTargetSelector,
    limit: Option<u32>,
) -> Result<Json<InviteLinkListResponse>, ApiError> {
    let now = OffsetDateTime::now_utc();
    let links = list_links(
        state.invites.as_ref(),
        state.sessions.as_ref(),
        state.rooms.as_ref(),
        InviteLinkQuery {
            actor: principal.actor_id,
            target,
            limit,
        },
        now,
    )
    .await?;
    Ok(Json(InviteLinkListResponse {
        links: links
            .iter()
            .map(|link| InviteLinkResponse::from_link(link, now))
            .collect(),
    }))
}
