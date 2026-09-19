use super::dto::{AddMemberRequest, MemberResponse};
use crate::http::{
    auth::Authenticated,
    error::ApiError,
    extractor::{ValidatedJson, ValidatedPath},
    state::{AppState, RoomStoreState},
};
use axum::{Json, extract::State, http::StatusCode};
use mahfel_application::rooms::{add_member, remove_member};
use mahfel_domain::{RoomId, RoomRole, UserId};
use uuid::Uuid;

pub(super) async fn add<R: RoomStoreState>(
    State(state): State<AppState<R>>,
    Authenticated(principal): Authenticated,
    ValidatedPath(room_id): ValidatedPath<Uuid>,
    ValidatedJson(body): ValidatedJson<AddMemberRequest>,
) -> Result<(StatusCode, Json<MemberResponse>), ApiError> {
    let member = add_member(
        &state.rooms,
        RoomId::from_uuid(room_id),
        principal.user_id,
        UserId::from_uuid(body.user_id),
        body.role.unwrap_or(RoomRole::Member),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(member.into())))
}

pub(super) async fn remove<R: RoomStoreState>(
    State(state): State<AppState<R>>,
    Authenticated(principal): Authenticated,
    ValidatedPath((room_id, user_id)): ValidatedPath<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    remove_member(
        &state.rooms,
        RoomId::from_uuid(room_id),
        principal.user_id,
        UserId::from_uuid(user_id),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
