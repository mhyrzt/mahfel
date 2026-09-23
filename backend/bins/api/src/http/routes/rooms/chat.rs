use crate::http::{
    auth::Authenticated,
    error::ApiError,
    extractor::{ValidatedJson, ValidatedPath},
    state::AppState,
};
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use mahfel_application::chat::{
    CreateChatMessage, DeleteChatMessage, EditChatMessage, ListChatMessages, check_chat_post_rate,
    create_chat_message, delete_chat_message, edit_chat_message, list_chat_messages,
};
use mahfel_domain::{ActorId, ChatMessage, ChatMessageCursor, ChatMessageId, RoomId};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateChatMessageRequest {
    pub(crate) content: String,
    #[serde(default)]
    pub(crate) mentions: Vec<Uuid>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ChatMessageResponse {
    pub(crate) id: String,
    pub(crate) channel_id: String,
    pub(crate) author_id: String,
    /// Null for soft-deleted messages; prior text is preserved in revision audit data.
    pub(crate) content: Option<String>,
    pub(crate) created_at: OffsetDateTime,
    pub(crate) edited_at: Option<OffsetDateTime>,
    pub(crate) deleted_at: Option<OffsetDateTime>,
    pub(crate) parent_id: Option<String>,
}

impl From<ChatMessage> for ChatMessageResponse {
    fn from(message: ChatMessage) -> Self {
        Self {
            id: message.id.to_string(),
            channel_id: message.channel_id.to_string(),
            author_id: message.author_id.to_string(),
            content: message.deleted_at.is_none().then_some(message.content),
            created_at: message.created_at,
            edited_at: message.edited_at,
            deleted_at: message.deleted_at,
            parent_id: message.parent_id.map(|id| id.to_string()),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ChatMessageHistoryResponse {
    pub(crate) messages: Vec<ChatMessageResponse>,
    pub(crate) next_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChatMessageHistoryQuery {
    pub(crate) cursor: Option<String>,
    pub(crate) limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChatMessagePath {
    pub(crate) room_id: Uuid,
    pub(crate) message_id: Uuid,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct EditChatMessageRequest {
    pub(crate) content: String,
    #[serde(default)]
    pub(crate) mentions: Vec<Uuid>,
}

#[utoipa::path(
    get,
    path = "/v1/rooms/{room_id}/messages",
    summary = "List room messages",
    operation_id = "listRoomMessages",
    tag = "chat",
    params(
        ("room_id" = Uuid, Path, description = "Room chat channel"),
        ("cursor" = Option<String>, Query, description = "Opaque cursor from the previous page"),
        ("limit" = Option<u32>, Query, description = "Page size, 1-100; defaults to 50"),
    ),
    responses(
        (status = 200, description = "Newest-first chat messages", body = ChatMessageHistoryResponse),
        (status = 400, description = "Invalid cursor"),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Room not found or caller is not a member"),
    ),
)]
pub(crate) async fn list_messages(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(room_id): ValidatedPath<Uuid>,
    Query(query): Query<ChatMessageHistoryQuery>,
) -> Result<Json<ChatMessageHistoryResponse>, ApiError> {
    let before = query.cursor.as_deref().map(decode_cursor).transpose()?;
    let page = list_chat_messages(
        state.chat.as_ref(),
        ListChatMessages {
            room_id: RoomId::from_uuid(room_id),
            actor_id: principal.actor_id,
            before,
            limit: query.limit.unwrap_or(50),
        },
    )
    .await?;
    Ok(Json(ChatMessageHistoryResponse {
        messages: page
            .messages
            .into_iter()
            .map(ChatMessageResponse::from)
            .collect(),
        next_cursor: page.next_cursor.map(encode_cursor).transpose()?,
    }))
}

#[utoipa::path(
    post,
    path = "/v1/rooms/{room_id}/messages",
    summary = "Send a room message",
    operation_id = "createRoomMessage",
    tag = "chat",
    params(("room_id" = Uuid, Path, description = "Room chat channel")),
    request_body = CreateChatMessageRequest,
    responses(
        (status = 201, description = "Message created", body = ChatMessageResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Room not found or caller is not a member"),
        (status = 422, description = "Message content is invalid"),
        (status = 429, description = "Chat posting limit exceeded"),
    ),
)]
pub(crate) async fn create_message(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(room_id): ValidatedPath<Uuid>,
    ValidatedJson(body): ValidatedJson<CreateChatMessageRequest>,
) -> Result<(StatusCode, Json<ChatMessageResponse>), ApiError> {
    check_chat_post_rate(
        state.code_index.as_ref(),
        RoomId::from_uuid(room_id),
        principal.actor_id,
    )
    .await?;
    let message = create_chat_message(
        state.chat.as_ref(),
        CreateChatMessage {
            room_id: mahfel_domain::RoomId::from_uuid(room_id),
            author_id: principal.actor_id,
            content: body.content,
            parent_id: None,
            mentions: body.mentions.into_iter().map(ActorId::from_uuid).collect(),
        },
    )
    .await?;
    state.realtime.publish_chat_wakeup().await;
    Ok((StatusCode::CREATED, Json(message.into())))
}

#[utoipa::path(
    patch,
    path = "/v1/rooms/{room_id}/messages/{message_id}",
    summary = "Edit a room message",
    operation_id = "editRoomMessage",
    tag = "chat",
    params(
        ("room_id" = Uuid, Path, description = "Room identifier"),
        ("message_id" = Uuid, Path, description = "Message identifier"),
    ),
    request_body = EditChatMessageRequest,
    responses(
        (status = 200, description = "Message edited; authors may edit within 15 minutes of creation", body = ChatMessageResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Message not found or caller is not its author"),
        (status = 409, description = "Message is deleted or its edit window expired"),
        (status = 422, description = "Message content is invalid"),
    ),
)]
pub(crate) async fn edit_message(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(path): ValidatedPath<ChatMessagePath>,
    ValidatedJson(body): ValidatedJson<EditChatMessageRequest>,
) -> Result<Json<ChatMessageResponse>, ApiError> {
    let message = edit_chat_message(
        state.chat.as_ref(),
        EditChatMessage {
            room_id: RoomId::from_uuid(path.room_id),
            actor_id: principal.actor_id,
            message_id: ChatMessageId::from_uuid(path.message_id),
            content: body.content,
            mentions: body.mentions.into_iter().map(ActorId::from_uuid).collect(),
        },
    )
    .await?;
    state.realtime.publish_chat_wakeup().await;
    Ok(Json(message.into()))
}

#[utoipa::path(
    delete,
    path = "/v1/rooms/{room_id}/messages/{message_id}",
    summary = "Delete a room message",
    operation_id = "deleteRoomMessage",
    tag = "chat",
    params(
        ("room_id" = Uuid, Path, description = "Room identifier"),
        ("message_id" = Uuid, Path, description = "Message identifier"),
    ),
    responses(
        (status = 200, description = "Message soft-deleted by its author within 15 minutes or by a room manager", body = ChatMessageResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Message not found or caller cannot delete it"),
    ),
)]
pub(crate) async fn delete_message(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(path): ValidatedPath<ChatMessagePath>,
) -> Result<Json<ChatMessageResponse>, ApiError> {
    let message = delete_chat_message(
        state.chat.as_ref(),
        DeleteChatMessage {
            room_id: RoomId::from_uuid(path.room_id),
            actor_id: principal.actor_id,
            message_id: ChatMessageId::from_uuid(path.message_id),
        },
    )
    .await?;
    state.realtime.publish_chat_wakeup().await;
    Ok(Json(message.into()))
}

pub(super) fn encode_cursor(cursor: ChatMessageCursor) -> Result<String, ApiError> {
    let timestamp = cursor
        .created_at
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(URL_SAFE_NO_PAD.encode(format!("v1|{timestamp}|{}", cursor.message_id)))
}

pub(super) fn decode_cursor(cursor: &str) -> Result<ChatMessageCursor, ApiError> {
    if cursor.len() > 256 {
        return Err(ApiError::bad_request("invalid chat cursor"));
    }
    let decoded = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| ApiError::bad_request("invalid chat cursor"))?;
    let decoded =
        std::str::from_utf8(&decoded).map_err(|_| ApiError::bad_request("invalid chat cursor"))?;
    let mut fields = decoded.split('|');
    let (Some("v1"), Some(timestamp), Some(message_id), None) =
        (fields.next(), fields.next(), fields.next(), fields.next())
    else {
        return Err(ApiError::bad_request("invalid chat cursor"));
    };
    let created_at =
        OffsetDateTime::parse(timestamp, &time::format_description::well_known::Rfc3339)
            .map_err(|_| ApiError::bad_request("invalid chat cursor"))?;
    let message_id =
        Uuid::parse_str(message_id).map_err(|_| ApiError::bad_request("invalid chat cursor"))?;
    Ok(ChatMessageCursor::new(
        created_at,
        ChatMessageId::from_uuid(message_id),
    ))
}

#[cfg(test)]
mod tests {
    use super::{ChatMessageResponse, decode_cursor, encode_cursor};
    use mahfel_domain::{
        ActorId, ChatChannelId, ChatMessage, ChatMessageCursor, ChatMessageDraft, ChatMessageId,
    };
    use time::OffsetDateTime;

    #[test]
    fn cursor_round_trips_timestamp_and_message_id() {
        let cursor = ChatMessageCursor::new(OffsetDateTime::UNIX_EPOCH, ChatMessageId::new());

        let encoded = encode_cursor(cursor.clone()).expect("encode valid cursor");

        assert_eq!(decode_cursor(&encoded).expect("decode cursor"), cursor);
    }

    #[test]
    fn malformed_or_oversized_cursor_is_rejected() {
        assert!(decode_cursor("bad").is_err());
        assert!(decode_cursor(&"x".repeat(257)).is_err());
    }

    #[test]
    fn deleted_message_response_does_not_expose_body() {
        let now = OffsetDateTime::UNIX_EPOCH;
        let draft = ChatMessageDraft::new(
            ChatMessageId::new(),
            ActorId::new(),
            "private removed text".to_owned(),
            now,
        )
        .expect("valid message");
        let message = ChatMessage {
            deleted_at: Some(now),
            ..ChatMessage::from_draft(ChatChannelId::new(), draft)
        };

        assert_eq!(ChatMessageResponse::from(message).content, None);
    }
}
