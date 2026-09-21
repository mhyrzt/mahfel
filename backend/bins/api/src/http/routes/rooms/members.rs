use super::dto::{AddMemberRequest, MemberResponse, RoomRoleDto};
use crate::http::{
    auth::Authenticated,
    error::ApiError,
    extractor::{ValidatedJson, ValidatedPath},
    state::{AppState, RoomStoreState},
};
use axum::{Json, extract::State, http::StatusCode};
use mahfel_application::rooms::{add_member, remove_member};
use mahfel_domain::{ActorId, RoomId};
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/v1/rooms/{room_id}/members",
    tag = "rooms",
    params(("room_id" = String, Path, description = "Room identifier (UUID)")),
    request_body = AddMemberRequest,
    responses(
        (status = 201, description = "Member added", body = MemberResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Caller is not a manager"),
        (status = 404, description = "Room, caller, or target user not found"),
        (status = 409, description = "User is already a member"),
    ),
)]
pub(crate) async fn add<R: RoomStoreState>(
    State(state): State<AppState<R>>,
    Authenticated(principal): Authenticated,
    ValidatedPath(room_id): ValidatedPath<Uuid>,
    ValidatedJson(body): ValidatedJson<AddMemberRequest>,
) -> Result<(StatusCode, Json<MemberResponse>), ApiError> {
    let member = add_member(
        &state.rooms,
        RoomId::from_uuid(room_id),
        principal.actor_id,
        ActorId::from_uuid(body.user_id),
        body.role.unwrap_or(RoomRoleDto::Member).into(),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(member.into())))
}

#[utoipa::path(
    delete,
    path = "/v1/rooms/{room_id}/members/{user_id}",
    tag = "rooms",
    params(
        ("room_id" = String, Path, description = "Room identifier (UUID)"),
        ("user_id" = String, Path, description = "Member user identifier (UUID)"),
    ),
    responses(
        (status = 204, description = "Member removed"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Caller is not a manager"),
        (status = 404, description = "Room, caller, or member not found"),
        (status = 409, description = "The last manager cannot be removed"),
    ),
)]
pub(crate) async fn remove<R: RoomStoreState>(
    State(state): State<AppState<R>>,
    Authenticated(principal): Authenticated,
    ValidatedPath((room_id, user_id)): ValidatedPath<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    remove_member(
        &state.rooms,
        RoomId::from_uuid(room_id),
        principal.actor_id,
        ActorId::from_uuid(user_id),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
