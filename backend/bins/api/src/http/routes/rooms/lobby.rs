use crate::http::{
    auth::Authenticated, error::ApiError, extractor::ValidatedPath, state::AppState,
};
use axum::{Json, extract::State, http::StatusCode};
use mahfel_application::admission::{AdmissionDecision, admit, reject, view_room_lobby};
use mahfel_domain::{ActorId, AdmissionTarget, RoomId};
use serde::Serialize;
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct RoomLobbyEntryResponse {
    request_id: String,
    actor_id: String,
    origin: String,
    #[serde(with = "crate::http::routes::rfc3339")]
    requested_at: OffsetDateTime,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct RoomLobbyResponse {
    room_id: String,
    entries: Vec<RoomLobbyEntryResponse>,
}

#[utoipa::path(
    get,
    path = "/v1/rooms/{room_id}/lobby",
    operation_id = "getRoomLobby",
    tag = "admission",
    params(("room_id" = String, Path)),
    responses((status = 200, body = RoomLobbyResponse), (status = 401), (status = 403), (status = 404)),
)]
pub(crate) async fn get_lobby(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(room_id): ValidatedPath<Uuid>,
) -> Result<Json<RoomLobbyResponse>, ApiError> {
    let room_id = RoomId::from_uuid(room_id);
    let lobby = view_room_lobby(
        state.admissions.as_ref(),
        state.rooms.as_ref(),
        principal.actor_id,
        room_id,
        None,
    )
    .await?;
    Ok(Json(RoomLobbyResponse {
        room_id: room_id.to_string(),
        entries: lobby
            .entries
            .into_iter()
            .map(|entry| RoomLobbyEntryResponse {
                request_id: entry.request_id.to_string(),
                actor_id: entry.actor_id.to_string(),
                origin: entry.origin.as_str().to_owned(),
                requested_at: entry.requested_at,
            })
            .collect(),
    }))
}

#[utoipa::path(
    post,
    path = "/v1/rooms/{room_id}/lobby/{actor_id}/admit",
    operation_id = "admitRoomPrincipal",
    tag = "admission",
    params(("room_id" = String, Path), ("actor_id" = String, Path)),
    responses((status = 204), (status = 401), (status = 403), (status = 404), (status = 409)),
)]
pub(crate) async fn admit_principal(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath((room_id, actor_id)): ValidatedPath<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    admit(
        state.realtime.stores.participants.as_ref(),
        state.participation.as_ref(),
        state.admissions.as_ref(),
        state.sessions.as_ref(),
        state.rooms.as_ref(),
        AdmissionDecision {
            actor: principal.actor_id,
            target: AdmissionTarget::Room(RoomId::from_uuid(room_id)),
            subject: ActorId::from_uuid(actor_id),
        },
        OffsetDateTime::now_utc(),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/v1/rooms/{room_id}/lobby/{actor_id}/reject",
    operation_id = "rejectRoomPrincipal",
    tag = "admission",
    params(("room_id" = String, Path), ("actor_id" = String, Path)),
    responses((status = 204), (status = 401), (status = 403), (status = 404), (status = 409)),
)]
pub(crate) async fn reject_principal(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath((room_id, actor_id)): ValidatedPath<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    reject(
        state.admissions.as_ref(),
        state.sessions.as_ref(),
        state.rooms.as_ref(),
        AdmissionDecision {
            actor: principal.actor_id,
            target: AdmissionTarget::Room(RoomId::from_uuid(room_id)),
            subject: ActorId::from_uuid(actor_id),
        },
        OffsetDateTime::now_utc(),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
