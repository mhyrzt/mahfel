use mahfel_application::ports::{ChatStore, StoreError};
use mahfel_domain::{
    ActorId, CHAT_AUTHOR_MUTATION_WINDOW, ChatChannel, ChatChannelId, ChatMessage,
    ChatMessageCursor, ChatMessageDraft, ChatMessageId, ChatMessagePage, RoomId,
};
use sqlx::PgPool;
use time::OffsetDateTime;

use super::sqlx_store_error;

#[derive(Clone)]
pub struct PostgresChatStore {
    pool: PgPool,
}

impl PostgresChatStore {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ChatStore for PostgresChatStore {
    fn create_message<'a>(
        &'a self,
        channel: &'a ChatChannel,
        draft: &'a ChatMessageDraft,
    ) -> mahfel_application::ports::BoxFuture<'a, Result<ChatMessage, StoreError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(sqlx_store_error)?;

            let authorized: Option<i32> = sqlx::query_scalar(
                "SELECT 1 \
                 FROM mahfel.rooms room \
                 INNER JOIN mahfel.room_memberships membership \
                   ON membership.room_id = room.id \
                 WHERE room.id = $1 AND room.status = 'active' \
                   AND membership.actor_id = $2 \
                 FOR SHARE OF room, membership",
            )
            .bind(channel.room_id.as_uuid())
            .bind(draft.author_id.as_uuid())
            .fetch_optional(&mut *tx)
            .await
            .map_err(sqlx_store_error)?;

            if authorized.is_none() {
                return Err(StoreError::NotFound("chat channel".to_owned()));
            }

            let inserted_channel: Option<uuid::Uuid> = sqlx::query_scalar(
                "INSERT INTO mahfel.chat_channels (id, room_id, created_at) \
                 VALUES ($1, $2, $3) \
                 ON CONFLICT (room_id) DO NOTHING \
                 RETURNING id",
            )
            .bind(channel.id.as_uuid())
            .bind(channel.room_id.as_uuid())
            .bind(channel.created_at)
            .fetch_optional(&mut *tx)
            .await
            .map_err(sqlx_store_error)?;

            let channel_id = match inserted_channel {
                Some(id) => id,
                None => sqlx::query_scalar::<_, uuid::Uuid>(
                    "SELECT id FROM mahfel.chat_channels WHERE room_id = $1",
                )
                .bind(channel.room_id.as_uuid())
                .fetch_one(&mut *tx)
                .await
                .map_err(sqlx_store_error)?,
            };

            sqlx::query(
                "INSERT INTO mahfel.chat_messages \
                 (id, channel_id, author_actor_id, body, created_at) \
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(draft.id.as_uuid())
            .bind(channel_id)
            .bind(draft.author_id.as_uuid())
            .bind(&draft.content)
            .bind(draft.created_at)
            .execute(&mut *tx)
            .await
            .map_err(sqlx_store_error)?;

            tx.commit().await.map_err(sqlx_store_error)?;
            Ok(ChatMessage::from_draft(
                ChatChannelId::from_uuid(channel_id),
                draft.clone(),
            ))
        })
    }

    fn list_messages<'a>(
        &'a self,
        room_id: RoomId,
        actor_id: ActorId,
        before: Option<&'a ChatMessageCursor>,
        limit: u32,
    ) -> mahfel_application::ports::BoxFuture<'a, Result<ChatMessagePage, StoreError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(sqlx_store_error)?;
            let authorized: Option<uuid::Uuid> = sqlx::query_scalar(
                "SELECT room.id \
                 FROM mahfel.rooms room \
                 INNER JOIN mahfel.room_memberships membership \
                   ON membership.room_id = room.id \
                 WHERE room.id = $1 AND room.status = 'active' \
                   AND membership.actor_id = $2 \
                 FOR SHARE OF room, membership",
            )
            .bind(room_id.as_uuid())
            .bind(actor_id.as_uuid())
            .fetch_optional(&mut *tx)
            .await
            .map_err(sqlx_store_error)?;
            if authorized.is_none() {
                return Err(StoreError::NotFound("chat channel".to_owned()));
            }

            let channel_id: Option<uuid::Uuid> =
                sqlx::query_scalar("SELECT id FROM mahfel.chat_channels WHERE room_id = $1")
                    .bind(room_id.as_uuid())
                    .fetch_optional(&mut *tx)
                    .await
                    .map_err(sqlx_store_error)?;
            let Some(channel_id) = channel_id else {
                tx.commit().await.map_err(sqlx_store_error)?;
                return Ok(ChatMessagePage {
                    messages: Vec::new(),
                    next_cursor: None,
                });
            };

            let before_created_at = before.map(|cursor| cursor.created_at);
            let before_id = before.map(|cursor| cursor.message_id.as_uuid());
            let fetch_limit = limit.saturating_add(1);
            let rows = sqlx::query_as::<_, ChatMessageRow>(
                "SELECT id, channel_id, author_actor_id, body, created_at, edited_at, deleted_at \
                 FROM mahfel.chat_messages \
                 WHERE channel_id = $1 \
                   AND ($2::TIMESTAMPTZ IS NULL OR (created_at, id) < ($2, $3)) \
                 ORDER BY created_at DESC, id DESC \
                 LIMIT $4",
            )
            .bind(channel_id)
            .bind(before_created_at)
            .bind(before_id)
            .bind(i64::from(fetch_limit))
            .fetch_all(&mut *tx)
            .await
            .map_err(sqlx_store_error)?;

            let has_more = rows.len() > limit as usize;
            let mut messages = rows
                .into_iter()
                .map(chat_message_from_row)
                .collect::<Vec<_>>();
            if has_more {
                messages.truncate(limit as usize);
            }
            let next_cursor = if has_more {
                messages
                    .last()
                    .map(|last| ChatMessageCursor::new(last.created_at, last.id))
            } else {
                None
            };
            tx.commit().await.map_err(sqlx_store_error)?;
            Ok(ChatMessagePage {
                messages,
                next_cursor,
            })
        })
    }

    fn edit_message<'a>(
        &'a self,
        room_id: RoomId,
        actor_id: ActorId,
        message_id: ChatMessageId,
        content: &'a str,
        now: OffsetDateTime,
    ) -> mahfel_application::ports::BoxFuture<'a, Result<ChatMessage, StoreError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(sqlx_store_error)?;
            let row = sqlx::query_as::<_, LockedChatMessageRow>(
                "SELECT message.id, message.channel_id, message.author_actor_id, message.body, \
                 message.created_at, message.edited_at, message.deleted_at, membership.member_role \
                 FROM mahfel.chat_messages message \
                 INNER JOIN mahfel.chat_channels channel ON channel.id = message.channel_id \
                 INNER JOIN mahfel.rooms room ON room.id = channel.room_id \
                 INNER JOIN mahfel.room_memberships membership \
                   ON membership.room_id = room.id AND membership.actor_id = $3 \
                 WHERE channel.room_id = $1 AND message.id = $2 AND room.status = 'active' \
                 FOR UPDATE OF message, room, membership",
            )
            .bind(room_id.as_uuid())
            .bind(message_id.as_uuid())
            .bind(actor_id.as_uuid())
            .fetch_optional(&mut *tx)
            .await
            .map_err(sqlx_store_error)?
            .ok_or_else(|| StoreError::NotFound("chat message".to_owned()))?;

            let message = row.message();
            if message.deleted_at.is_some() {
                return Err(StoreError::Conflict("message is deleted".to_owned()));
            }
            if message.author_id != actor_id {
                return Err(StoreError::NotFound("chat message".to_owned()));
            }
            if now > message.created_at + CHAT_AUTHOR_MUTATION_WINDOW {
                return Err(StoreError::Conflict(
                    "message edit window expired".to_owned(),
                ));
            }

            insert_revision(
                &mut tx,
                message_id,
                actor_id,
                &message.content,
                "edited",
                now,
            )
            .await?;
            let updated = sqlx::query_as::<_, ChatMessageRow>(
                "UPDATE mahfel.chat_messages \
                 SET body = $2, edited_at = $3 \
                 WHERE id = $1 \
                 RETURNING id, channel_id, author_actor_id, body, created_at, edited_at, deleted_at",
            )
            .bind(message_id.as_uuid())
            .bind(content)
            .bind(now)
            .fetch_one(&mut *tx)
            .await
            .map_err(sqlx_store_error)?;
            tx.commit().await.map_err(sqlx_store_error)?;
            Ok(chat_message_from_row(updated))
        })
    }

    fn delete_message<'a>(
        &'a self,
        room_id: RoomId,
        actor_id: ActorId,
        message_id: ChatMessageId,
        now: OffsetDateTime,
    ) -> mahfel_application::ports::BoxFuture<'a, Result<ChatMessage, StoreError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(sqlx_store_error)?;
            let row = sqlx::query_as::<_, LockedChatMessageRow>(
                "SELECT message.id, message.channel_id, message.author_actor_id, message.body, \
                 message.created_at, message.edited_at, message.deleted_at, membership.member_role \
                 FROM mahfel.chat_messages message \
                 INNER JOIN mahfel.chat_channels channel ON channel.id = message.channel_id \
                 INNER JOIN mahfel.rooms room ON room.id = channel.room_id \
                 INNER JOIN mahfel.room_memberships membership \
                   ON membership.room_id = room.id AND membership.actor_id = $3 \
                 WHERE channel.room_id = $1 AND message.id = $2 AND room.status = 'active' \
                 FOR UPDATE OF message, room, membership",
            )
            .bind(room_id.as_uuid())
            .bind(message_id.as_uuid())
            .bind(actor_id.as_uuid())
            .fetch_optional(&mut *tx)
            .await
            .map_err(sqlx_store_error)?
            .ok_or_else(|| StoreError::NotFound("chat message".to_owned()))?;

            let message = row.message();
            if let Some(deleted_at) = message.deleted_at {
                tx.commit().await.map_err(sqlx_store_error)?;
                return Ok(ChatMessage {
                    deleted_at: Some(deleted_at),
                    ..message
                });
            }
            let is_manager = row.member_role == "manager";
            let is_author_in_window = message.author_id == actor_id
                && now <= message.created_at + CHAT_AUTHOR_MUTATION_WINDOW;
            if !is_manager && !is_author_in_window {
                return Err(StoreError::NotFound("chat message".to_owned()));
            }

            insert_revision(
                &mut tx,
                message_id,
                actor_id,
                &message.content,
                "deleted",
                now,
            )
            .await?;
            let updated = sqlx::query_as::<_, ChatMessageRow>(
                "UPDATE mahfel.chat_messages SET deleted_at = $2 \
                 WHERE id = $1 \
                 RETURNING id, channel_id, author_actor_id, body, created_at, edited_at, deleted_at",
            )
            .bind(message_id.as_uuid())
            .bind(now)
            .fetch_one(&mut *tx)
            .await
            .map_err(sqlx_store_error)?;
            tx.commit().await.map_err(sqlx_store_error)?;
            Ok(chat_message_from_row(updated))
        })
    }
}

type ChatMessageRow = (
    uuid::Uuid,
    uuid::Uuid,
    uuid::Uuid,
    String,
    OffsetDateTime,
    Option<OffsetDateTime>,
    Option<OffsetDateTime>,
);

#[derive(sqlx::FromRow)]
struct LockedChatMessageRow {
    id: uuid::Uuid,
    channel_id: uuid::Uuid,
    author_actor_id: uuid::Uuid,
    body: String,
    created_at: OffsetDateTime,
    edited_at: Option<OffsetDateTime>,
    deleted_at: Option<OffsetDateTime>,
    member_role: String,
}

impl LockedChatMessageRow {
    fn message(&self) -> ChatMessage {
        ChatMessage {
            id: ChatMessageId::from_uuid(self.id),
            channel_id: ChatChannelId::from_uuid(self.channel_id),
            author_id: ActorId::from_uuid(self.author_actor_id),
            content: self.body.clone(),
            created_at: self.created_at,
            edited_at: self.edited_at,
            deleted_at: self.deleted_at,
        }
    }
}

fn chat_message_from_row(row: ChatMessageRow) -> ChatMessage {
    ChatMessage {
        id: ChatMessageId::from_uuid(row.0),
        channel_id: ChatChannelId::from_uuid(row.1),
        author_id: ActorId::from_uuid(row.2),
        content: row.3,
        created_at: row.4,
        edited_at: row.5,
        deleted_at: row.6,
    }
}

async fn insert_revision(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    message_id: ChatMessageId,
    editor_id: ActorId,
    prior_body: &str,
    kind: &str,
    now: OffsetDateTime,
) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO mahfel.chat_message_revisions \
         (id, message_id, editor_actor_id, prior_body, revision_kind, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(uuid::Uuid::new_v4())
    .bind(message_id.as_uuid())
    .bind(editor_id.as_uuid())
    .bind(prior_body)
    .bind(kind)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(sqlx_store_error)?;
    Ok(())
}
