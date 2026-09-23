use crate::http::{
    auth::Authenticated,
    error::ApiError,
    extractor::{ValidatedJson, ValidatedPath},
    state::AppState,
};
use axum::{Json, extract::State, http::StatusCode};
use mahfel_application::chat::{CreateChatMessage, create_chat_message};
use mahfel_domain::ChatMessage;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateChatMessageRequest {
    pub(crate) content: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ChatMessageResponse {
    pub(crate) id: String,
    pub(crate) channel_id: String,
    pub(crate) author_id: String,
    pub(crate) content: String,
    pub(crate) created_at: OffsetDateTime,
    pub(crate) deleted_at: Option<OffsetDateTime>,
}

impl From<ChatMessage> for ChatMessageResponse {
    fn from(message: ChatMessage) -> Self {
        Self {
            id: message.id.to_string(),
            channel_id: message.channel_id.to_string(),
            author_id: message.author_id.to_string(),
            content: message.content,
            created_at: message.created_at,
            deleted_at: message.deleted_at,
        }
    }
}

#[utoipa::path(
    post,
    path = "/v1/rooms/{room_id}/messages",
    operation_id = "createRoomMessage",
    tag = "chat",
    params(("room_id" = Uuid, Path, description = "Room chat channel")),
    request_body = CreateChatMessageRequest,
    responses(
        (status = 201, description = "Message created", body = ChatMessageResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Room not found or caller is not a member"),
        (status = 422, description = "Message content is invalid"),
    ),
)]
pub(crate) async fn create_message(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(room_id): ValidatedPath<Uuid>,
    ValidatedJson(body): ValidatedJson<CreateChatMessageRequest>,
) -> Result<(StatusCode, Json<ChatMessageResponse>), ApiError> {
    let message = create_chat_message(
        state.chat.as_ref(),
        CreateChatMessage {
            room_id: mahfel_domain::RoomId::from_uuid(room_id),
            author_id: principal.actor_id,
            content: body.content,
        },
    )
    .await?;
    Ok((StatusCode::CREATED, Json(message.into())))
}
