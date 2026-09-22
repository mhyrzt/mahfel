use crate::DatabaseError;
use crate::stores::{sqlx_store_error, store_error};
use mahfel_application::ports::{BlockStore, BoxFuture, StoreError};
use mahfel_domain::{ActorId, Block, BlockId, BlockScope, RoomId, SessionId};
use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresBlockStore {
    pool: PgPool,
}

impl PostgresBlockStore {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct BlockRow {
    id: Uuid,
    room_id: Option<Uuid>,
    session_id: Option<Uuid>,
    actor_id: Uuid,
    blocked_by: Uuid,
    reason: Option<String>,
    blocked_at: OffsetDateTime,
    expires_at: Option<OffsetDateTime>,
    lifted_at: Option<OffsetDateTime>,
    lifted_by: Option<Uuid>,
}

impl TryFrom<BlockRow> for Block {
    type Error = DatabaseError;

    fn try_from(row: BlockRow) -> Result<Self, Self::Error> {
        // The scope CHECK guarantees exactly one target is present, so a row
        // that violates it is a data defect rather than a block we can serve.
        let scope = match (row.room_id, row.session_id) {
            (Some(room_id), _) => BlockScope::Room(RoomId::from_uuid(room_id)),
            (None, Some(session_id)) => BlockScope::Session(SessionId::from_uuid(session_id)),
            (None, None) => {
                return Err(DatabaseError::InvalidStoredValue(
                    "block names neither a room nor a session",
                ));
            }
        };
        Ok(Block {
            id: BlockId::from_uuid(row.id),
            scope,
            actor_id: ActorId::from_uuid(row.actor_id),
            blocked_by: ActorId::from_uuid(row.blocked_by),
            reason: row.reason,
            blocked_at: row.blocked_at,
            expires_at: row.expires_at,
            lifted_at: row.lifted_at,
            lifted_by: row.lifted_by.map(ActorId::from_uuid),
        })
    }
}

const SELECT_COLUMNS: &str = "id, room_id, session_id, actor_id, blocked_by, reason, \
                              blocked_at, expires_at, lifted_at, lifted_by";

impl BlockStore for PostgresBlockStore {
    fn get_active(
        &self,
        scope: BlockScope,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<Option<Block>, StoreError>> {
        Box::pin(async move {
            // Active means neither lifted nor expired. An expired block was
            // never lifted, it simply ran out, so it must not be returned.
            let query = format!(
                "SELECT {SELECT_COLUMNS} FROM mahfel.blocks \
                 WHERE room_id IS NOT DISTINCT FROM $1 \
                   AND session_id IS NOT DISTINCT FROM $2 \
                   AND actor_id = $3 \
                   AND lifted_at IS NULL \
                   AND (expires_at IS NULL OR expires_at > $4) \
                 LIMIT 1"
            );
            let row = sqlx::query_as::<_, BlockRow>(sqlx::AssertSqlSafe(query))
                .bind(scope.room_id().map(RoomId::as_uuid))
                .bind(scope.session_id().map(SessionId::as_uuid))
                .bind(actor_id.as_uuid())
                .bind(OffsetDateTime::now_utc())
                .fetch_optional(&self.pool)
                .await
                .map_err(sqlx_store_error)?;
            row.map(Block::try_from).transpose().map_err(store_error)
        })
    }

    fn insert<'a>(&'a self, block: &'a Block) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            // Retire any lapsed block for the same target first. A partial unique
            // index cannot express `expires_at > now()` because `now()` is not
            // immutable, so an expired row would otherwise keep occupying the
            // active slot and block a legitimate new one. Marking it lifted with
            // no principal records that it ran out rather than being cleared.
            let retire = match block.scope {
                BlockScope::Room(room_id) => sqlx::query(
                    "UPDATE mahfel.blocks SET lifted_at = $3, lifted_by = NULL \
                     WHERE room_id = $1 AND actor_id = $2 \
                       AND lifted_at IS NULL AND expires_at IS NOT NULL \
                       AND expires_at <= $3",
                )
                .bind(room_id.as_uuid())
                .bind(block.actor_id.as_uuid())
                .bind(block.blocked_at),
                BlockScope::Session(session_id) => sqlx::query(
                    "UPDATE mahfel.blocks SET lifted_at = $3, lifted_by = NULL \
                     WHERE session_id = $1 AND actor_id = $2 \
                       AND lifted_at IS NULL AND expires_at IS NOT NULL \
                       AND expires_at <= $3",
                )
                .bind(session_id.as_uuid())
                .bind(block.actor_id.as_uuid())
                .bind(block.blocked_at),
            };
            retire.execute(&self.pool).await.map_err(sqlx_store_error)?;

            sqlx::query(
                "INSERT INTO mahfel.blocks \
                 (id, room_id, session_id, actor_id, blocked_by, reason, blocked_at, \
                  expires_at, lifted_at, lifted_by) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
            )
            .bind(block.id.as_uuid())
            .bind(block.scope.room_id().map(RoomId::as_uuid))
            .bind(block.scope.session_id().map(SessionId::as_uuid))
            .bind(block.actor_id.as_uuid())
            .bind(block.blocked_by.as_uuid())
            .bind(block.reason.as_deref())
            .bind(block.blocked_at)
            .bind(block.expires_at)
            .bind(block.lifted_at)
            .bind(block.lifted_by.map(ActorId::as_uuid))
            .execute(&self.pool)
            .await
            .map_err(sqlx_store_error)?;
            Ok(())
        })
    }

    fn extend(
        &self,
        id: BlockId,
        expires_at: Option<OffsetDateTime>,
    ) -> BoxFuture<'_, Result<bool, StoreError>> {
        Box::pin(async move {
            // Only an active block is extended, so a lift is never undone by a
            // later extension.
            let result = sqlx::query(
                "UPDATE mahfel.blocks SET expires_at = $2 \
                 WHERE id = $1 AND lifted_at IS NULL",
            )
            .bind(id.as_uuid())
            .bind(expires_at)
            .execute(&self.pool)
            .await
            .map_err(sqlx_store_error)?;
            Ok(result.rows_affected() == 1)
        })
    }

    fn lift(
        &self,
        id: BlockId,
        lifted_by: ActorId,
        lifted_at: OffsetDateTime,
    ) -> BoxFuture<'_, Result<bool, StoreError>> {
        Box::pin(async move {
            // Conditional so a duplicate lift cannot overwrite who lifted it.
            let result = sqlx::query(
                "UPDATE mahfel.blocks SET lifted_at = $2, lifted_by = $3 \
                 WHERE id = $1 AND lifted_at IS NULL",
            )
            .bind(id.as_uuid())
            .bind(lifted_at)
            .bind(lifted_by.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(sqlx_store_error)?;
            Ok(result.rows_affected() == 1)
        })
    }

    fn list_active_for_room(
        &self,
        room_id: RoomId,
        limit: u32,
    ) -> BoxFuture<'_, Result<Vec<Block>, StoreError>> {
        Box::pin(async move {
            let query = format!(
                "SELECT {SELECT_COLUMNS} FROM mahfel.blocks \
                 WHERE room_id = $1 AND lifted_at IS NULL \
                   AND (expires_at IS NULL OR expires_at > $2) \
                 ORDER BY blocked_at, id LIMIT $3"
            );
            let rows = sqlx::query_as::<_, BlockRow>(sqlx::AssertSqlSafe(query))
                .bind(room_id.as_uuid())
                .bind(OffsetDateTime::now_utc())
                .bind(i64::from(limit))
                .fetch_all(&self.pool)
                .await
                .map_err(sqlx_store_error)?;
            rows.into_iter()
                .map(|row| Block::try_from(row).map_err(store_error))
                .collect()
        })
    }

    fn list_active_for_session(
        &self,
        session_id: SessionId,
        limit: u32,
    ) -> BoxFuture<'_, Result<Vec<Block>, StoreError>> {
        Box::pin(async move {
            let query = format!(
                "SELECT {SELECT_COLUMNS} FROM mahfel.blocks \
                 WHERE session_id = $1 AND lifted_at IS NULL \
                   AND (expires_at IS NULL OR expires_at > $2) \
                 ORDER BY blocked_at, id LIMIT $3"
            );
            let rows = sqlx::query_as::<_, BlockRow>(sqlx::AssertSqlSafe(query))
                .bind(session_id.as_uuid())
                .bind(OffsetDateTime::now_utc())
                .bind(i64::from(limit))
                .fetch_all(&self.pool)
                .await
                .map_err(sqlx_store_error)?;
            rows.into_iter()
                .map(|row| Block::try_from(row).map_err(store_error))
                .collect()
        })
    }
}
