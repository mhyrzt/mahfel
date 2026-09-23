use mahfel_application::ports::{ChatStore, StoreError};
use mahfel_domain::{
    ActorId, CHAT_AUTHOR_MUTATION_WINDOW, ChatChannel, ChatChannelId, ChatEvent, ChatEventKind,
    ChatMessage, ChatMessageCursor, ChatMessageDraft, ChatMessageId, ChatMessagePage, RoomId,
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
    fn read_marker(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
    ) -> mahfel_application::ports::BoxFuture<'_, Result<i64, StoreError>> {
        Box::pin(super::chat_interactions::read_marker(
            &self.pool, room_id, actor_id,
        ))
    }

    fn advance_read_marker(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
        sequence: i64,
        now: OffsetDateTime,
    ) -> mahfel_application::ports::BoxFuture<'_, Result<i64, StoreError>> {
        Box::pin(super::chat_interactions::advance_read_marker(
            &self.pool, room_id, actor_id, sequence, now,
        ))
    }

    fn list_replies<'a>(
        &'a self,
        room_id: RoomId,
        actor_id: ActorId,
        parent_id: ChatMessageId,
        before: Option<&'a ChatMessageCursor>,
        limit: u32,
    ) -> mahfel_application::ports::BoxFuture<'a, Result<ChatMessagePage, StoreError>> {
        Box::pin(super::chat_interactions::list_replies(
            &self.pool, room_id, actor_id, parent_id, before, limit,
        ))
    }

    fn list_mentions(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
        message_id: ChatMessageId,
    ) -> mahfel_application::ports::BoxFuture<'_, Result<Vec<ActorId>, StoreError>> {
        Box::pin(super::chat_interactions::list_mentions(
            &self.pool, room_id, actor_id, message_id,
        ))
    }

    fn set_reaction<'a>(
        &'a self,
        room_id: RoomId,
        actor_id: ActorId,
        message_id: ChatMessageId,
        emoji: &'a str,
        now: OffsetDateTime,
    ) -> mahfel_application::ports::BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(super::chat_interactions::set_reaction(
            &self.pool, room_id, actor_id, message_id, emoji, now,
        ))
    }

    fn remove_reaction(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
        message_id: ChatMessageId,
        now: OffsetDateTime,
    ) -> mahfel_application::ports::BoxFuture<'_, Result<(), StoreError>> {
        Box::pin(super::chat_interactions::remove_reaction(
            &self.pool, room_id, actor_id, message_id, now,
        ))
    }

    fn list_reactions(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
        message_id: ChatMessageId,
    ) -> mahfel_application::ports::BoxFuture<
        '_,
        Result<mahfel_application::ports::ChatReactionSnapshot, StoreError>,
    > {
        Box::pin(super::chat_interactions::list_reactions(
            &self.pool, room_id, actor_id, message_id,
        ))
    }

    fn event_head(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
    ) -> mahfel_application::ports::BoxFuture<'_, Result<i64, StoreError>> {
        Box::pin(async move {
            let head: Option<i64> = sqlx::query_scalar(
                "SELECT COALESCE(channel.event_sequence, 0) \
                 FROM mahfel.rooms room \
                 INNER JOIN mahfel.room_memberships membership \
                   ON membership.room_id = room.id \
                 LEFT JOIN mahfel.chat_channels channel ON channel.room_id = room.id \
                 WHERE room.id = $1 AND room.status = 'active' AND membership.actor_id = $2",
            )
            .bind(room_id.as_uuid())
            .bind(actor_id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(sqlx_store_error)?;
            head.ok_or_else(|| StoreError::NotFound("chat channel".to_owned()))
        })
    }

    fn list_events(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
        after_sequence: i64,
        limit: u32,
    ) -> mahfel_application::ports::BoxFuture<'_, Result<Vec<ChatEvent>, StoreError>> {
        Box::pin(async move {
            if after_sequence < 0 || limit == 0 || limit > 100 {
                return Err(StoreError::Unexpected(
                    "invalid chat event cursor or limit".to_owned(),
                ));
            }
            let mut tx = self.pool.begin().await.map_err(sqlx_store_error)?;
            let authorized: Option<uuid::Uuid> = sqlx::query_scalar(
                "SELECT room.id FROM mahfel.rooms room \
                 INNER JOIN mahfel.room_memberships membership \
                   ON membership.room_id = room.id \
                 WHERE room.id = $1 AND room.status = 'active' AND membership.actor_id = $2 \
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

            let rows = sqlx::query_as::<_, ChatEventRow>(
                "SELECT event.id, event.event_seq AS sequence, event.kind, event.message_id, \
                 event.channel_id, event.author_actor_id, event.body, \
                 event.message_created_at, event.message_edited_at, event.message_deleted_at, \
                 event.parent_id, event.reaction_actor_id, event.reaction_emoji \
                 FROM mahfel.chat_events event \
                 INNER JOIN mahfel.chat_channels channel ON channel.id = event.channel_id \
                 WHERE channel.room_id = $1 AND event.event_seq > $2 \
                 ORDER BY event.event_seq ASC LIMIT $3",
            )
            .bind(room_id.as_uuid())
            .bind(after_sequence)
            .bind(i64::from(limit))
            .fetch_all(&mut *tx)
            .await
            .map_err(sqlx_store_error)?;
            tx.commit().await.map_err(sqlx_store_error)?;
            rows.into_iter()
                .map(|row| row.into_event(room_id))
                .collect()
        })
    }

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

            if let Some(parent_id) = draft.parent_id {
                let parent: Option<uuid::Uuid> = sqlx::query_scalar(
                    "SELECT id FROM mahfel.chat_messages \
                     WHERE id = $1 AND channel_id = $2 AND parent_id IS NULL \
                       AND deleted_at IS NULL FOR SHARE",
                )
                .bind(parent_id.as_uuid())
                .bind(channel_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(sqlx_store_error)?;
                if parent.is_none() {
                    return Err(StoreError::Conflict("invalid thread parent".to_owned()));
                }
            }

            sqlx::query(
                "INSERT INTO mahfel.chat_messages \
                 (id, channel_id, author_actor_id, body, created_at, parent_id) \
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(draft.id.as_uuid())
            .bind(channel_id)
            .bind(draft.author_id.as_uuid())
            .bind(&draft.content)
            .bind(draft.created_at)
            .bind(draft.parent_id.map(|id| id.as_uuid()))
            .execute(&mut *tx)
            .await
            .map_err(sqlx_store_error)?;

            replace_mentions(&mut tx, channel.room_id, draft.id, &draft.mentions).await?;

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
                "SELECT id, channel_id, author_actor_id, body, created_at, edited_at, deleted_at, parent_id \
                 FROM mahfel.chat_messages \
                 WHERE channel_id = $1 \
                   AND parent_id IS NULL \
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
        mentions: &'a [ActorId],
        now: OffsetDateTime,
    ) -> mahfel_application::ports::BoxFuture<'a, Result<ChatMessage, StoreError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(sqlx_store_error)?;
            let row = sqlx::query_as::<_, LockedChatMessageRow>(
                "SELECT message.id, message.channel_id, message.author_actor_id, message.body, \
                 message.created_at, message.edited_at, message.deleted_at, message.parent_id, membership.member_role \
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
                 RETURNING id, channel_id, author_actor_id, body, created_at, edited_at, deleted_at, parent_id",
            )
            .bind(message_id.as_uuid())
            .bind(content)
            .bind(now)
            .fetch_one(&mut *tx)
            .await
            .map_err(sqlx_store_error)?;
            replace_mentions(&mut tx, room_id, message_id, mentions).await?;
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
                 message.created_at, message.edited_at, message.deleted_at, message.parent_id, membership.member_role \
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
                 RETURNING id, channel_id, author_actor_id, body, created_at, edited_at, deleted_at, parent_id",
            )
            .bind(message_id.as_uuid())
            .bind(now)
            .fetch_one(&mut *tx)
            .await
            .map_err(sqlx_store_error)?;
            sqlx::query(
                "UPDATE mahfel.files SET status = 'deleting', updated_at = $2 \
                 WHERE chat_message_id = $1 AND status IN ('pending','quarantined','ready')",
            )
            .bind(message_id.as_uuid())
            .bind(now)
            .execute(&mut *tx)
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
    Option<uuid::Uuid>,
);

#[derive(sqlx::FromRow)]
struct ChatEventRow {
    id: uuid::Uuid,
    sequence: i64,
    kind: String,
    message_id: uuid::Uuid,
    channel_id: uuid::Uuid,
    author_actor_id: uuid::Uuid,
    body: Option<String>,
    message_created_at: OffsetDateTime,
    message_edited_at: Option<OffsetDateTime>,
    message_deleted_at: Option<OffsetDateTime>,
    parent_id: Option<uuid::Uuid>,
    reaction_actor_id: Option<uuid::Uuid>,
    reaction_emoji: Option<String>,
}

impl ChatEventRow {
    fn into_event(self, room_id: RoomId) -> Result<ChatEvent, StoreError> {
        let kind = match self.kind.as_str() {
            "created" => ChatEventKind::Created,
            "edited" => ChatEventKind::Edited,
            "deleted" => ChatEventKind::Deleted,
            "reaction_set" => ChatEventKind::ReactionSet,
            "reaction_removed" => ChatEventKind::ReactionRemoved,
            _ => return Err(StoreError::Unexpected("unknown chat event kind".to_owned())),
        };
        Ok(ChatEvent {
            id: self.id,
            sequence: self.sequence,
            room_id,
            kind,
            message: ChatMessage {
                id: ChatMessageId::from_uuid(self.message_id),
                channel_id: ChatChannelId::from_uuid(self.channel_id),
                author_id: ActorId::from_uuid(self.author_actor_id),
                content: self.body.unwrap_or_default(),
                created_at: self.message_created_at,
                edited_at: self.message_edited_at,
                deleted_at: self.message_deleted_at,
                parent_id: self.parent_id.map(ChatMessageId::from_uuid),
            },
            reaction_actor_id: self.reaction_actor_id.map(ActorId::from_uuid),
            reaction_emoji: self.reaction_emoji,
        })
    }
}

#[derive(sqlx::FromRow)]
struct LockedChatMessageRow {
    id: uuid::Uuid,
    channel_id: uuid::Uuid,
    author_actor_id: uuid::Uuid,
    body: String,
    created_at: OffsetDateTime,
    edited_at: Option<OffsetDateTime>,
    deleted_at: Option<OffsetDateTime>,
    parent_id: Option<uuid::Uuid>,
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
            parent_id: self.parent_id.map(ChatMessageId::from_uuid),
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
        parent_id: row.7.map(ChatMessageId::from_uuid),
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

async fn replace_mentions(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    room_id: RoomId,
    message_id: ChatMessageId,
    mentions: &[ActorId],
) -> Result<(), StoreError> {
    if mentions.len() > 20 {
        return Err(StoreError::Conflict("too many mentions".to_owned()));
    }
    let ids = mentions.iter().map(|id| id.as_uuid()).collect::<Vec<_>>();
    let members = sqlx::query_scalar::<_, uuid::Uuid>(
        "SELECT actor_id FROM mahfel.room_memberships \
         WHERE room_id = $1 AND actor_id = ANY($2) FOR SHARE",
    )
    .bind(room_id.as_uuid())
    .bind(&ids)
    .fetch_all(&mut **tx)
    .await
    .map_err(sqlx_store_error)?;
    if members.len() != ids.len() {
        return Err(StoreError::Conflict("invalid mention".to_owned()));
    }
    sqlx::query("DELETE FROM mahfel.chat_mentions WHERE message_id = $1")
        .bind(message_id.as_uuid())
        .execute(&mut **tx)
        .await
        .map_err(sqlx_store_error)?;
    for actor_id in ids {
        sqlx::query("INSERT INTO mahfel.chat_mentions (message_id, actor_id) VALUES ($1, $2)")
            .bind(message_id.as_uuid())
            .bind(actor_id)
            .execute(&mut **tx)
            .await
            .map_err(sqlx_store_error)?;
    }
    Ok(())
}
