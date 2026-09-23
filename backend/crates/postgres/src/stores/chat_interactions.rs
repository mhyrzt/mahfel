use mahfel_application::ports::{ChatReactionSnapshot, StoreError};
use mahfel_domain::{
    ActorId, CHAT_REACTION_EMOJIS, ChatChannelId, ChatMessage, ChatMessageCursor, ChatMessageId,
    ChatMessagePage, RoomId,
};
use sqlx::{PgPool, Postgres, Transaction};
use time::OffsetDateTime;

use super::sqlx_store_error;

async fn authorized_room(
    tx: &mut Transaction<'_, Postgres>,
    room_id: RoomId,
    actor_id: ActorId,
) -> Result<(), StoreError> {
    let found: Option<uuid::Uuid> = sqlx::query_scalar(
        "SELECT room.id FROM mahfel.rooms room \
         INNER JOIN mahfel.room_memberships membership ON membership.room_id = room.id \
         WHERE room.id = $1 AND room.status = 'active' AND membership.actor_id = $2 \
         FOR SHARE OF room, membership",
    )
    .bind(room_id.as_uuid())
    .bind(actor_id.as_uuid())
    .fetch_optional(&mut **tx)
    .await
    .map_err(sqlx_store_error)?;
    if found.is_none() {
        return Err(StoreError::NotFound("chat channel".to_owned()));
    }
    Ok(())
}

pub(super) async fn read_marker(
    pool: &PgPool,
    room_id: RoomId,
    actor_id: ActorId,
) -> Result<i64, StoreError> {
    let mut tx = pool.begin().await.map_err(sqlx_store_error)?;
    authorized_room(&mut tx, room_id, actor_id).await?;
    let sequence = sqlx::query_scalar::<_, i64>(
        "SELECT marker.event_seq FROM mahfel.chat_read_markers marker \
         INNER JOIN mahfel.chat_channels channel ON channel.id = marker.channel_id \
         WHERE channel.room_id = $1 AND marker.actor_id = $2",
    )
    .bind(room_id.as_uuid())
    .bind(actor_id.as_uuid())
    .fetch_optional(&mut *tx)
    .await
    .map_err(sqlx_store_error)?
    .unwrap_or(0);
    tx.commit().await.map_err(sqlx_store_error)?;
    Ok(sequence)
}

pub(super) async fn advance_read_marker(
    pool: &PgPool,
    room_id: RoomId,
    actor_id: ActorId,
    sequence: i64,
    now: OffsetDateTime,
) -> Result<i64, StoreError> {
    if sequence < 0 {
        return Err(StoreError::Conflict("invalid read sequence".to_owned()));
    }
    let mut tx = pool.begin().await.map_err(sqlx_store_error)?;
    authorized_room(&mut tx, room_id, actor_id).await?;
    let channel: Option<(uuid::Uuid, i64)> = sqlx::query_as(
        "SELECT id, event_sequence FROM mahfel.chat_channels \
         WHERE room_id = $1 FOR SHARE",
    )
    .bind(room_id.as_uuid())
    .fetch_optional(&mut *tx)
    .await
    .map_err(sqlx_store_error)?;
    let Some((channel_id, head)) = channel else {
        if sequence == 0 {
            return Ok(0);
        }
        return Err(StoreError::Conflict(
            "read sequence exceeds channel head".to_owned(),
        ));
    };
    if sequence > head {
        return Err(StoreError::Conflict(
            "read sequence exceeds channel head".to_owned(),
        ));
    }
    let stored: i64 = sqlx::query_scalar(
        "INSERT INTO mahfel.chat_read_markers AS marker (channel_id, actor_id, event_seq, updated_at) \
         VALUES ($1, $2, $3, $4) \
         ON CONFLICT (channel_id, actor_id) DO UPDATE \
         SET event_seq = GREATEST(marker.event_seq, EXCLUDED.event_seq), \
             updated_at = CASE WHEN EXCLUDED.event_seq > marker.event_seq \
                 THEN EXCLUDED.updated_at ELSE marker.updated_at END \
         RETURNING event_seq",
    )
    .bind(channel_id)
    .bind(actor_id.as_uuid())
    .bind(sequence)
    .bind(now)
    .fetch_one(&mut *tx)
    .await
    .map_err(sqlx_store_error)?;
    tx.commit().await.map_err(sqlx_store_error)?;
    Ok(stored)
}

type MessageRow = (
    uuid::Uuid,
    uuid::Uuid,
    uuid::Uuid,
    String,
    OffsetDateTime,
    Option<OffsetDateTime>,
    Option<OffsetDateTime>,
    Option<uuid::Uuid>,
);

fn message(row: MessageRow) -> ChatMessage {
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

async fn visible_message(
    tx: &mut Transaction<'_, Postgres>,
    room_id: RoomId,
    actor_id: ActorId,
    message_id: ChatMessageId,
    live_only: bool,
) -> Result<uuid::Uuid, StoreError> {
    let channel_id: Option<uuid::Uuid> = sqlx::query_scalar(
        "SELECT message.channel_id FROM mahfel.chat_messages message \
         INNER JOIN mahfel.chat_channels channel ON channel.id = message.channel_id \
         INNER JOIN mahfel.rooms room ON room.id = channel.room_id \
         INNER JOIN mahfel.room_memberships membership \
           ON membership.room_id = room.id AND membership.actor_id = $2 \
         WHERE room.id = $1 AND room.status = 'active' AND message.id = $3 \
           AND (NOT $4 OR message.deleted_at IS NULL) \
         FOR SHARE OF room, membership, message",
    )
    .bind(room_id.as_uuid())
    .bind(actor_id.as_uuid())
    .bind(message_id.as_uuid())
    .bind(live_only)
    .fetch_optional(&mut **tx)
    .await
    .map_err(sqlx_store_error)?;
    channel_id.ok_or_else(|| StoreError::NotFound("chat message".to_owned()))
}

pub(super) async fn list_replies(
    pool: &PgPool,
    room_id: RoomId,
    actor_id: ActorId,
    parent_id: ChatMessageId,
    before: Option<&ChatMessageCursor>,
    limit: u32,
) -> Result<ChatMessagePage, StoreError> {
    if limit == 0 || limit > 100 {
        return Err(StoreError::Conflict("invalid reply page size".to_owned()));
    }
    let mut tx = pool.begin().await.map_err(sqlx_store_error)?;
    visible_message(&mut tx, room_id, actor_id, parent_id, false).await?;
    let is_root: bool =
        sqlx::query_scalar("SELECT parent_id IS NULL FROM mahfel.chat_messages WHERE id = $1")
            .bind(parent_id.as_uuid())
            .fetch_one(&mut *tx)
            .await
            .map_err(sqlx_store_error)?;
    if !is_root {
        return Err(StoreError::Conflict(
            "nested threads are not supported".to_owned(),
        ));
    }
    let rows = sqlx::query_as::<_, MessageRow>(
        "SELECT id, channel_id, author_actor_id, body, created_at, edited_at, deleted_at, parent_id \
         FROM mahfel.chat_messages WHERE parent_id = $1 \
           AND ($2::TIMESTAMPTZ IS NULL OR (created_at, id) < ($2, $3)) \
         ORDER BY created_at DESC, id DESC LIMIT $4",
    )
    .bind(parent_id.as_uuid())
    .bind(before.map(|cursor| cursor.created_at))
    .bind(before.map(|cursor| cursor.message_id.as_uuid()))
    .bind(i64::from(limit) + 1)
    .fetch_all(&mut *tx)
    .await
    .map_err(sqlx_store_error)?;
    let has_more = rows.len() > limit as usize;
    let mut messages = rows.into_iter().map(message).collect::<Vec<_>>();
    if has_more {
        messages.truncate(limit as usize);
    }
    let next_cursor = has_more
        .then(|| messages.last())
        .flatten()
        .map(|last| ChatMessageCursor::new(last.created_at, last.id));
    tx.commit().await.map_err(sqlx_store_error)?;
    Ok(ChatMessagePage {
        messages,
        next_cursor,
    })
}

pub(super) async fn list_mentions(
    pool: &PgPool,
    room_id: RoomId,
    actor_id: ActorId,
    message_id: ChatMessageId,
) -> Result<Vec<ActorId>, StoreError> {
    let mut tx = pool.begin().await.map_err(sqlx_store_error)?;
    visible_message(&mut tx, room_id, actor_id, message_id, false).await?;
    let ids = sqlx::query_scalar::<_, uuid::Uuid>(
        "SELECT actor_id FROM mahfel.chat_mentions WHERE message_id = $1 ORDER BY actor_id",
    )
    .bind(message_id.as_uuid())
    .fetch_all(&mut *tx)
    .await
    .map_err(sqlx_store_error)?;
    tx.commit().await.map_err(sqlx_store_error)?;
    Ok(ids.into_iter().map(ActorId::from_uuid).collect())
}

pub(super) async fn list_reactions(
    pool: &PgPool,
    room_id: RoomId,
    actor_id: ActorId,
    message_id: ChatMessageId,
) -> Result<ChatReactionSnapshot, StoreError> {
    let mut tx = pool.begin().await.map_err(sqlx_store_error)?;
    visible_message(&mut tx, room_id, actor_id, message_id, false).await?;
    let counts = sqlx::query_as::<_, (String, i64)>(
        "SELECT emoji, COUNT(*) FROM mahfel.chat_reactions \
         WHERE message_id = $1 GROUP BY emoji ORDER BY emoji",
    )
    .bind(message_id.as_uuid())
    .fetch_all(&mut *tx)
    .await
    .map_err(sqlx_store_error)?;
    let own = sqlx::query_scalar::<_, String>(
        "SELECT emoji FROM mahfel.chat_reactions WHERE message_id = $1 AND actor_id = $2",
    )
    .bind(message_id.as_uuid())
    .bind(actor_id.as_uuid())
    .fetch_optional(&mut *tx)
    .await
    .map_err(sqlx_store_error)?;
    tx.commit().await.map_err(sqlx_store_error)?;
    Ok((counts, own))
}

pub(super) async fn set_reaction(
    pool: &PgPool,
    room_id: RoomId,
    actor_id: ActorId,
    message_id: ChatMessageId,
    emoji: &str,
    now: OffsetDateTime,
) -> Result<(), StoreError> {
    if !CHAT_REACTION_EMOJIS.contains(&emoji) {
        return Err(StoreError::Conflict(
            "unsupported reaction emoji".to_owned(),
        ));
    }
    let mut tx = pool.begin().await.map_err(sqlx_store_error)?;
    let channel_id = visible_message(&mut tx, room_id, actor_id, message_id, true).await?;
    let changed: Option<uuid::Uuid> = sqlx::query_scalar(
        "INSERT INTO mahfel.chat_reactions (message_id, actor_id, emoji, updated_at) \
         VALUES ($1, $2, $3, $4) \
         ON CONFLICT (message_id, actor_id) DO UPDATE \
         SET emoji = EXCLUDED.emoji, updated_at = EXCLUDED.updated_at \
         WHERE chat_reactions.emoji IS DISTINCT FROM EXCLUDED.emoji \
         RETURNING message_id",
    )
    .bind(message_id.as_uuid())
    .bind(actor_id.as_uuid())
    .bind(emoji)
    .bind(now)
    .fetch_optional(&mut *tx)
    .await
    .map_err(sqlx_store_error)?;
    if changed.is_some() {
        append_reaction_event(
            &mut tx,
            channel_id,
            message_id,
            actor_id,
            "reaction_set",
            Some(emoji),
        )
        .await?;
    }
    tx.commit().await.map_err(sqlx_store_error)?;
    Ok(())
}

pub(super) async fn remove_reaction(
    pool: &PgPool,
    room_id: RoomId,
    actor_id: ActorId,
    message_id: ChatMessageId,
    now: OffsetDateTime,
) -> Result<(), StoreError> {
    let mut tx = pool.begin().await.map_err(sqlx_store_error)?;
    let channel_id = visible_message(&mut tx, room_id, actor_id, message_id, false).await?;
    let removed: Option<String> = sqlx::query_scalar(
        "DELETE FROM mahfel.chat_reactions WHERE message_id = $1 AND actor_id = $2 \
         RETURNING emoji",
    )
    .bind(message_id.as_uuid())
    .bind(actor_id.as_uuid())
    .fetch_optional(&mut *tx)
    .await
    .map_err(sqlx_store_error)?;
    if removed.is_some() {
        append_reaction_event(
            &mut tx,
            channel_id,
            message_id,
            actor_id,
            "reaction_removed",
            None,
        )
        .await?;
    }
    let _ = now;
    tx.commit().await.map_err(sqlx_store_error)?;
    Ok(())
}

async fn append_reaction_event(
    tx: &mut Transaction<'_, Postgres>,
    channel_id: uuid::Uuid,
    message_id: ChatMessageId,
    actor_id: ActorId,
    kind: &str,
    emoji: Option<&str>,
) -> Result<(), StoreError> {
    let sequence: i64 = sqlx::query_scalar(
        "UPDATE mahfel.chat_channels SET event_sequence = event_sequence + 1 \
         WHERE id = $1 RETURNING event_sequence",
    )
    .bind(channel_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(sqlx_store_error)?;
    sqlx::query(
        "INSERT INTO mahfel.chat_events (
           channel_id, event_seq, id, kind, message_id, author_actor_id, body,
           message_created_at, message_edited_at, message_deleted_at,
           occurred_at, parent_id, reaction_actor_id, reaction_emoji
         ) SELECT $1, $2, gen_random_uuid(), $3, message.id, message.author_actor_id,
           CASE WHEN message.deleted_at IS NULL THEN message.body ELSE NULL END,
           message.created_at, message.edited_at, message.deleted_at,
           clock_timestamp(), message.parent_id, $4, $5
         FROM mahfel.chat_messages message WHERE message.id = $6",
    )
    .bind(channel_id)
    .bind(sequence)
    .bind(kind)
    .bind(actor_id.as_uuid())
    .bind(emoji)
    .bind(message_id.as_uuid())
    .execute(&mut **tx)
    .await
    .map_err(sqlx_store_error)?;
    Ok(())
}
