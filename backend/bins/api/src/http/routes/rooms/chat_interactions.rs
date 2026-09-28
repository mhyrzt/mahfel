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
use mahfel_application::chat::{
    ChatOperationError, CreateChatMessage, check_chat_post_rate, create_chat_message,
};
use mahfel_domain::{ActorId, CHAT_REACTION_EMOJIS, ChatMessageId, RoomId};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;

use super::chat::{
    ChatMessageHistoryQuery, ChatMessageHistoryResponse, ChatMessagePath, ChatMessageResponse,
    CreateChatMessageRequest, decode_cursor, encode_cursor, with_attachments,
};

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ChatMentionsResponse {
    actor_ids: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct SetReactionRequest {
    emoji: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ReactionCountResponse {
    emoji: String,
    count: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ReactionsResponse {
    counts: Vec<ReactionCountResponse>,
    my_reaction: Option<String>,
}

#[utoipa::path(
    get,
    path = "/v1/rooms/{room_id}/messages/{message_id}/replies",
    summary = "List direct replies to a room message",
    operation_id = "listRoomMessageReplies",
    tag = "chat",
    params(("room_id" = Uuid, Path), ("message_id" = Uuid, Path),
        ("cursor" = Option<String>, Query), ("limit" = Option<u32>, Query)),
    responses((status = 200, body = ChatMessageHistoryResponse), (status = 400), (status = 404)),
)]
pub(crate) async fn list_replies(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(path): ValidatedPath<ChatMessagePath>,
    Query(query): Query<ChatMessageHistoryQuery>,
) -> Result<Json<ChatMessageHistoryResponse>, ApiError> {
    let before = query.cursor.as_deref().map(decode_cursor).transpose()?;
    let limit = query.limit.unwrap_or(50);
    if !(1..=100).contains(&limit) {
        return Err(ApiError::bad_request("invalid reply page size"));
    }
    let page = state
        .chat
        .list_replies(
            RoomId::from_uuid(path.room_id),
            principal.actor_id,
            ChatMessageId::from_uuid(path.message_id),
            before.as_ref(),
            limit,
        )
        .await
        .map_err(ChatOperationError::from)?;
    Ok(Json(ChatMessageHistoryResponse {
        messages: with_attachments(&state, page.messages).await?,
        next_cursor: page.next_cursor.map(encode_cursor).transpose()?,
    }))
}

#[utoipa::path(
    post,
    path = "/v1/rooms/{room_id}/messages/{message_id}/replies",
    summary = "Reply to a root room message",
    operation_id = "createRoomMessageReply",
    tag = "chat",
    params(("room_id" = Uuid, Path), ("message_id" = Uuid, Path)),
    request_body = CreateChatMessageRequest,
    responses((status = 201, body = ChatMessageResponse), (status = 404), (status = 409), (status = 422), (status = 429)),
)]
pub(crate) async fn create_reply(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(path): ValidatedPath<ChatMessagePath>,
    ValidatedJson(body): ValidatedJson<CreateChatMessageRequest>,
) -> Result<(StatusCode, Json<ChatMessageResponse>), ApiError> {
    check_chat_post_rate(
        state.code_index.as_ref(),
        RoomId::from_uuid(path.room_id),
        principal.actor_id,
    )
    .await?;
    let message = create_chat_message(
        state.chat.as_ref(),
        CreateChatMessage {
            room_id: RoomId::from_uuid(path.room_id),
            author_id: principal.actor_id,
            content: body.content,
            parent_id: Some(ChatMessageId::from_uuid(path.message_id)),
            mentions: body.mentions.into_iter().map(ActorId::from_uuid).collect(),
            session_id: None,
        },
    )
    .await?;
    state.realtime.publish_chat_wakeup().await;
    Ok((StatusCode::CREATED, Json(message.into())))
}

#[utoipa::path(
    get,
    path = "/v1/rooms/{room_id}/messages/{message_id}/mentions",
    summary = "List structured mentions on a message",
    operation_id = "listRoomMessageMentions",
    tag = "chat",
    params(("room_id" = Uuid, Path), ("message_id" = Uuid, Path)),
    responses((status = 200, body = ChatMentionsResponse), (status = 404)),
)]
pub(crate) async fn list_mentions(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(path): ValidatedPath<ChatMessagePath>,
) -> Result<Json<ChatMentionsResponse>, ApiError> {
    let ids = state
        .chat
        .list_mentions(
            RoomId::from_uuid(path.room_id),
            principal.actor_id,
            ChatMessageId::from_uuid(path.message_id),
        )
        .await
        .map_err(ChatOperationError::from)?;
    Ok(Json(ChatMentionsResponse {
        actor_ids: ids.into_iter().map(|id| id.to_string()).collect(),
    }))
}

#[utoipa::path(
    get,
    path = "/v1/rooms/{room_id}/messages/{message_id}/reaction",
    summary = "Get reaction counts and the caller's reaction",
    operation_id = "getRoomMessageReactions",
    tag = "chat",
    params(("room_id" = Uuid, Path), ("message_id" = Uuid, Path)),
    responses((status = 200, body = ReactionsResponse), (status = 404)),
)]
pub(crate) async fn get_reactions(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(path): ValidatedPath<ChatMessagePath>,
) -> Result<Json<ReactionsResponse>, ApiError> {
    let (counts, my_reaction) = state
        .chat
        .list_reactions(
            RoomId::from_uuid(path.room_id),
            principal.actor_id,
            ChatMessageId::from_uuid(path.message_id),
        )
        .await
        .map_err(ChatOperationError::from)?;
    Ok(Json(ReactionsResponse {
        counts: counts
            .into_iter()
            .map(|(emoji, count)| ReactionCountResponse { emoji, count })
            .collect(),
        my_reaction,
    }))
}

#[utoipa::path(
    put,
    path = "/v1/rooms/{room_id}/messages/{message_id}/reaction",
    summary = "Set the caller's single emoji reaction",
    operation_id = "setRoomMessageReaction",
    tag = "chat",
    params(("room_id" = Uuid, Path), ("message_id" = Uuid, Path)),
    request_body = SetReactionRequest,
    responses((status = 204), (status = 404), (status = 422)),
)]
pub(crate) async fn set_reaction(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(path): ValidatedPath<ChatMessagePath>,
    ValidatedJson(body): ValidatedJson<SetReactionRequest>,
) -> Result<StatusCode, ApiError> {
    if !CHAT_REACTION_EMOJIS.contains(&body.emoji.as_str()) {
        return Err(ApiError::unprocessable("unsupported reaction emoji"));
    }
    state
        .chat
        .set_reaction(
            RoomId::from_uuid(path.room_id),
            principal.actor_id,
            ChatMessageId::from_uuid(path.message_id),
            &body.emoji,
            OffsetDateTime::now_utc(),
        )
        .await
        .map_err(ChatOperationError::from)?;
    state.realtime.publish_chat_wakeup().await;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/v1/rooms/{room_id}/messages/{message_id}/reaction",
    summary = "Remove the caller's reaction",
    operation_id = "removeRoomMessageReaction",
    tag = "chat",
    params(("room_id" = Uuid, Path), ("message_id" = Uuid, Path)),
    responses((status = 204), (status = 404)),
)]
pub(crate) async fn remove_reaction(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    ValidatedPath(path): ValidatedPath<ChatMessagePath>,
) -> Result<StatusCode, ApiError> {
    state
        .chat
        .remove_reaction(
            RoomId::from_uuid(path.room_id),
            principal.actor_id,
            ChatMessageId::from_uuid(path.message_id),
            OffsetDateTime::now_utc(),
        )
        .await
        .map_err(ChatOperationError::from)?;
    state.realtime.publish_chat_wakeup().await;
    Ok(StatusCode::NO_CONTENT)
}
