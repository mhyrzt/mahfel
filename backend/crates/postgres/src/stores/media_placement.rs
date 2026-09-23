use mahfel_application::ports::{BoxFuture, MediaPlacement, MediaPlacementStore, StoreError};
use mahfel_domain::{NodeId, RoomId, SessionId};
use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::stores::sqlx_store_error;

#[derive(Clone)]
pub struct PostgresMediaPlacementStore {
    pool: PgPool,
}

impl PostgresMediaPlacementStore {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct PlacementRow {
    session_id: Uuid,
    room_id: Uuid,
    node_id: String,
    max_rooms: i32,
    active_rooms: i64,
    reason: String,
    decided_at: OffsetDateTime,
}

impl TryFrom<PlacementRow> for MediaPlacement {
    type Error = StoreError;

    fn try_from(row: PlacementRow) -> Result<Self, Self::Error> {
        Ok(Self {
            session_id: SessionId::from_uuid(row.session_id),
            room_id: RoomId::from_uuid(row.room_id),
            node_id: NodeId::from_deployment(row.node_id)
                .ok_or_else(|| StoreError::Unexpected("invalid persisted node id".into()))?,
            max_rooms: u32::try_from(row.max_rooms).map_err(StoreError::unexpected)?,
            active_rooms: u64::try_from(row.active_rooms).map_err(StoreError::unexpected)?,
            reason: row.reason,
            decided_at: row.decided_at,
        })
    }
}

impl MediaPlacementStore for PostgresMediaPlacementStore {
    fn get(
        &self,
        session_id: SessionId,
    ) -> BoxFuture<'_, Result<Option<MediaPlacement>, StoreError>> {
        Box::pin(async move {
            sqlx::query_as::<_, PlacementRow>(
                "SELECT session_id, room_id, node_id, max_rooms, active_rooms, reason, decided_at \
                 FROM mahfel.media_placements WHERE session_id = $1",
            )
            .bind(session_id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(sqlx_store_error)?
            .map(MediaPlacement::try_from)
            .transpose()
        })
    }

    fn insert_or_get<'a>(
        &'a self,
        placement: &'a MediaPlacement,
    ) -> BoxFuture<'a, Result<MediaPlacement, StoreError>> {
        Box::pin(async move {
            sqlx::query(
                "INSERT INTO mahfel.media_placements \
                 (session_id, room_id, node_id, max_rooms, active_rooms, reason, decided_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7) \
                 ON CONFLICT (session_id) DO NOTHING",
            )
            .bind(placement.session_id.as_uuid())
            .bind(placement.room_id.as_uuid())
            .bind(placement.node_id.as_str())
            .bind(i32::try_from(placement.max_rooms).map_err(StoreError::unexpected)?)
            .bind(i64::try_from(placement.active_rooms).map_err(StoreError::unexpected)?)
            .bind(&placement.reason)
            .bind(placement.decided_at)
            .execute(&self.pool)
            .await
            .map_err(sqlx_store_error)?;
            self.get(placement.session_id)
                .await?
                .ok_or_else(|| StoreError::Unexpected("placement vanished after insert".into()))
        })
    }
}
