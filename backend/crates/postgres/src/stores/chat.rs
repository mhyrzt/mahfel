use mahfel_application::ports::{ChatStore, StoreError};
use mahfel_domain::{ChatChannel, ChatChannelId, ChatMessage, ChatMessageDraft};
use sqlx::PgPool;

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
}
