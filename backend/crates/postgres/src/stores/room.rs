use crate::DatabaseError;
use crate::stores::{sqlx_store_error, store_error};
use mahfel_application::ports::{BoxFuture, RoomStore, StoreError};
use mahfel_domain::{ActorId, Room, RoomId, RoomMember, RoomRole, RoomTitle, Session};
use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresRoomStore {
    pool: PgPool,
}

impl PostgresRoomStore {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Insert a development room, its memberships, and scheduled sessions
    /// atomically. Existing rows are left intact on repeated seed runs.
    pub async fn seed_development_room(
        &self,
        room: &Room,
        members: &[RoomMember],
        sessions: &[Session],
    ) -> Result<(), StoreError> {
        let mut tx = self.pool.begin().await.map_err(sqlx_store_error)?;
        sqlx::query(
            "INSERT INTO mahfel.rooms \
             (id, title, created_by, visibility, admission, status, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $7) ON CONFLICT (id) DO NOTHING",
        )
        .bind(room.id.as_uuid())
        .bind(room.title.as_str())
        .bind(room.created_by.as_uuid())
        .bind(room.visibility.as_str())
        .bind(room.admission.as_str())
        .bind(room.status.as_str())
        .bind(room.created_at)
        .execute(&mut *tx)
        .await
        .map_err(sqlx_store_error)?;
        let stored_owner: Uuid =
            sqlx::query_scalar("SELECT created_by FROM mahfel.rooms WHERE id = $1")
                .bind(room.id.as_uuid())
                .fetch_one(&mut *tx)
                .await
                .map_err(sqlx_store_error)?;
        if stored_owner != room.created_by.as_uuid() {
            return Err(StoreError::Conflict(format!(
                "seed room {} belongs to another actor",
                room.id
            )));
        }
        for member in members {
            sqlx::query(
                "INSERT INTO mahfel.room_memberships \
                 (room_id, actor_id, member_role, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $4) ON CONFLICT (room_id, actor_id) DO NOTHING",
            )
            .bind(member.room_id.as_uuid())
            .bind(member.actor_id.as_uuid())
            .bind(member.role.as_str())
            .bind(member.created_at)
            .execute(&mut *tx)
            .await
            .map_err(sqlx_store_error)?;
        }
        for session in sessions {
            sqlx::query(
                "INSERT INTO mahfel.sessions \
                 (id, room_id, status, title, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, $5) ON CONFLICT (id) DO NOTHING",
            )
            .bind(session.id.as_uuid())
            .bind(session.room_id.as_uuid())
            .bind(session.status.as_str())
            .bind(session.title.as_ref().map(|title| title.as_str()))
            .bind(session.created_at)
            .execute(&mut *tx)
            .await
            .map_err(sqlx_store_error)?;
            let stored_room: Uuid =
                sqlx::query_scalar("SELECT room_id FROM mahfel.sessions WHERE id = $1")
                    .bind(session.id.as_uuid())
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(sqlx_store_error)?;
            if stored_room != room.id.as_uuid() {
                return Err(StoreError::Conflict(format!(
                    "seed session {} belongs to another room",
                    session.id
                )));
            }
        }
        tx.commit().await.map_err(sqlx_store_error)
    }
}

#[derive(FromRow)]
struct RoomRow {
    id: Uuid,
    title: String,
    created_by: Uuid,
    visibility: String,
    admission: String,
    status: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

#[derive(FromRow)]
struct RoomMemberRow {
    room_id: Uuid,
    actor_id: Uuid,
    #[sqlx(rename = "member_role")]
    role: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

impl RoomStore for PostgresRoomStore {
    fn get(&self, id: RoomId) -> BoxFuture<'_, Result<Option<Room>, StoreError>> {
        Box::pin(async move {
            let row = sqlx::query_as::<_, RoomRow>(
                "SELECT id, title, created_by, visibility, admission, status, created_at, updated_at \
                 FROM mahfel.rooms WHERE id = $1",
            )
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(sqlx_store_error)?;

            row.map(|row| Room::try_from(row).map_err(store_error))
                .transpose()
        })
    }

    fn insert<'a>(&'a self, room: &'a Room) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            sqlx::query(
                "INSERT INTO mahfel.rooms \
                 (id, title, created_by, visibility, admission, status, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $7)",
            )
            .bind(room.id.as_uuid())
            .bind(room.title.as_str())
            .bind(room.created_by.as_uuid())
            .bind(room.visibility.as_str())
            .bind(room.admission.as_str())
            .bind(room.status.as_str())
            .bind(room.created_at)
            .execute(&self.pool)
            .await
            .map_err(sqlx_store_error)?;
            Ok(())
        })
    }

    fn save<'a>(&'a self, room: &'a Room) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let result = sqlx::query(
                "UPDATE mahfel.rooms \
                 SET title = $2, visibility = $3, admission = $4, status = $5, updated_at = $6 \
                 WHERE id = $1",
            )
            .bind(room.id.as_uuid())
            .bind(room.title.as_str())
            .bind(room.visibility.as_str())
            .bind(room.admission.as_str())
            .bind(room.status.as_str())
            .bind(room.updated_at)
            .execute(&self.pool)
            .await
            .map_err(sqlx_store_error)?;

            if result.rows_affected() == 0 {
                return Err(StoreError::NotFound(room.id.to_string()));
            }
            Ok(())
        })
    }

    fn create_with_manager<'a>(
        &'a self,
        room: &'a Room,
        manager: &'a RoomMember,
    ) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(sqlx_store_error)?;
            sqlx::query(
                "INSERT INTO mahfel.rooms \
                 (id, title, created_by, visibility, admission, status, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $7)",
            )
            .bind(room.id.as_uuid())
            .bind(room.title.as_str())
            .bind(room.created_by.as_uuid())
            .bind(room.visibility.as_str())
            .bind(room.admission.as_str())
            .bind(room.status.as_str())
            .bind(room.created_at)
            .execute(&mut *tx)
            .await
            .map_err(sqlx_store_error)?;
            insert_membership(&mut tx, manager).await?;
            tx.commit().await.map_err(sqlx_store_error)?;
            Ok(())
        })
    }

    fn list_for_member(
        &self,
        actor_id: ActorId,
        limit: u64,
        offset: u64,
    ) -> BoxFuture<'_, Result<Vec<Room>, StoreError>> {
        Box::pin(async move {
            let rows = sqlx::query_as::<_, RoomRow>(
                "SELECT r.id, r.title, r.created_by, r.visibility, r.admission, r.status, \
                        r.created_at, r.updated_at \
                 FROM mahfel.rooms r \
                 JOIN mahfel.room_memberships m ON m.room_id = r.id \
                 WHERE m.actor_id = $1 \
                 ORDER BY r.created_at DESC, r.id \
                 LIMIT $2 OFFSET $3",
            )
            .bind(actor_id.as_uuid())
            .bind(i64::try_from(limit).unwrap_or(i64::MAX))
            .bind(i64::try_from(offset).unwrap_or(i64::MAX))
            .fetch_all(&self.pool)
            .await
            .map_err(sqlx_store_error)?;

            rows.into_iter()
                .map(|row| Room::try_from(row).map_err(store_error))
                .collect()
        })
    }

    fn get_member(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<Option<RoomMember>, StoreError>> {
        Box::pin(async move {
            let row = sqlx::query_as::<_, RoomMemberRow>(
                "SELECT room_id, actor_id, member_role, created_at, updated_at \
                 FROM mahfel.room_memberships WHERE room_id = $1 AND actor_id = $2",
            )
            .bind(room_id.as_uuid())
            .bind(actor_id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(sqlx_store_error)?;

            row.map(|row| RoomMember::try_from(row).map_err(store_error))
                .transpose()
        })
    }

    fn list_members(&self, room_id: RoomId) -> BoxFuture<'_, Result<Vec<RoomMember>, StoreError>> {
        Box::pin(async move {
            let rows = sqlx::query_as::<_, RoomMemberRow>(
                "SELECT room_id, actor_id, member_role, created_at, updated_at \
                 FROM mahfel.room_memberships WHERE room_id = $1 \
                 ORDER BY created_at, actor_id",
            )
            .bind(room_id.as_uuid())
            .fetch_all(&self.pool)
            .await
            .map_err(sqlx_store_error)?;

            rows.into_iter()
                .map(|row| RoomMember::try_from(row).map_err(store_error))
                .collect()
        })
    }

    fn add_member<'a>(&'a self, member: &'a RoomMember) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let result = sqlx::query(
                "INSERT INTO mahfel.room_memberships \
                 (room_id, actor_id, member_role, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $4) \
                 ON CONFLICT (room_id, actor_id) DO NOTHING",
            )
            .bind(member.room_id.as_uuid())
            .bind(member.actor_id.as_uuid())
            .bind(member.role.as_str())
            .bind(member.created_at)
            .execute(&self.pool)
            .await
            .map_err(sqlx_store_error)?;

            if result.rows_affected() == 0 {
                return Err(StoreError::Conflict(format!(
                    "user {} is already a member of room {}",
                    member.actor_id, member.room_id
                )));
            }
            Ok(())
        })
    }

    fn update_member<'a>(
        &'a self,
        member: &'a RoomMember,
    ) -> BoxFuture<'a, Result<bool, StoreError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(sqlx_store_error)?;
            let room = sqlx::query_scalar::<_, Uuid>(
                "SELECT id FROM mahfel.rooms WHERE id = $1 FOR UPDATE",
            )
            .bind(member.room_id.as_uuid())
            .fetch_optional(&mut *tx)
            .await
            .map_err(sqlx_store_error)?;
            if room.is_none() {
                return Ok(false);
            }
            let result = sqlx::query(
                "UPDATE mahfel.room_memberships SET member_role = $3, updated_at = $4 \
                 WHERE room_id = $1 AND actor_id = $2 \
                 AND (member_role <> 'manager' OR $3 = 'manager' OR \
                      (SELECT COUNT(*) FROM mahfel.room_memberships \
                       WHERE room_id = $1 AND member_role = 'manager') > 1)",
            )
            .bind(member.room_id.as_uuid())
            .bind(member.actor_id.as_uuid())
            .bind(member.role.as_str())
            .bind(member.updated_at)
            .execute(&mut *tx)
            .await
            .map_err(sqlx_store_error)?;
            tx.commit().await.map_err(sqlx_store_error)?;
            Ok(result.rows_affected() == 1)
        })
    }

    fn remove_member(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<bool, StoreError>> {
        Box::pin(async move {
            // Serialize membership changes for the room by locking its row, so the
            // manager count and the delete observe one consistent snapshot. The
            // domain check in the use case is defense in depth, not the guard.
            let mut tx = self.pool.begin().await.map_err(sqlx_store_error)?;
            let lock = sqlx::query_scalar::<_, Uuid>(
                "SELECT id FROM mahfel.rooms WHERE id = $1 FOR UPDATE",
            )
            .bind(room_id.as_uuid())
            .fetch_optional(&mut *tx)
            .await
            .map_err(sqlx_store_error)?;
            if lock.is_none() {
                return Ok(false);
            }

            let target = sqlx::query_as::<_, RoomMemberRow>(
                "SELECT room_id, actor_id, member_role, created_at, updated_at \
                 FROM mahfel.room_memberships WHERE room_id = $1 AND actor_id = $2",
            )
            .bind(room_id.as_uuid())
            .bind(actor_id.as_uuid())
            .fetch_optional(&mut *tx)
            .await
            .map_err(sqlx_store_error)?;
            let Some(target) = target else {
                return Ok(false);
            };
            let target = RoomMember::try_from(target).map_err(store_error)?;

            if target.role == RoomRole::Manager {
                let managers = sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM mahfel.room_memberships \
                     WHERE room_id = $1 AND member_role = 'manager'",
                )
                .bind(room_id.as_uuid())
                .fetch_one(&mut *tx)
                .await
                .map_err(sqlx_store_error)?;
                if managers <= 1 {
                    return Ok(false);
                }
            }

            let result = sqlx::query(
                "DELETE FROM mahfel.room_memberships WHERE room_id = $1 AND actor_id = $2",
            )
            .bind(room_id.as_uuid())
            .bind(actor_id.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(sqlx_store_error)?;
            tx.commit().await.map_err(sqlx_store_error)?;
            Ok(result.rows_affected() > 0)
        })
    }
}

/// Insert a membership using an existing transaction.
async fn insert_membership(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    member: &RoomMember,
) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO mahfel.room_memberships \
         (room_id, actor_id, member_role, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $4)",
    )
    .bind(member.room_id.as_uuid())
    .bind(member.actor_id.as_uuid())
    .bind(member.role.as_str())
    .bind(member.created_at)
    .execute(&mut **tx)
    .await
    .map_err(sqlx_store_error)?;
    Ok(())
}

impl TryFrom<RoomRow> for Room {
    type Error = DatabaseError;

    fn try_from(row: RoomRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: RoomId::from_uuid(row.id),
            title: RoomTitle::new(row.title.clone())
                .map_err(|_| DatabaseError::InvalidStoredValue("invalid room title"))?,
            created_by: ActorId::from_uuid(row.created_by),
            visibility: row.visibility.as_str().try_into()?,
            admission: row.admission.as_str().try_into()?,
            status: row.status.as_str().try_into()?,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<RoomMemberRow> for RoomMember {
    type Error = DatabaseError;

    fn try_from(row: RoomMemberRow) -> Result<Self, Self::Error> {
        Ok(Self {
            room_id: RoomId::from_uuid(row.room_id),
            actor_id: ActorId::from_uuid(row.actor_id),
            role: row.role.as_str().try_into()?,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
