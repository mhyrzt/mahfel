use crate::DatabaseError;
use mahfel_application::ports::StoreError;
use mahfel_domain::{
    ClassroomSession, ClassroomSessionId, DisplayName, Room, RoomId, RoomTitle, User, UserId,
};
use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresUserStore {
    pool: PgPool,
}

#[derive(Clone)]
pub struct PostgresRoomStore {
    pool: PgPool,
}

#[derive(Clone)]
pub struct PostgresSessionStore {
    pool: PgPool,
}

#[derive(FromRow)]
struct UserRow {
    id: Uuid,
    display_name: String,
    status: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

#[derive(FromRow)]
struct RoomRow {
    id: Uuid,
    title: String,
    created_by: Uuid,
    status: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

#[derive(FromRow)]
struct ClassroomSessionRow {
    id: Uuid,
    room_id: Uuid,
    status: String,
    scheduled_start_at: Option<OffsetDateTime>,
    started_at: Option<OffsetDateTime>,
    ended_at: Option<OffsetDateTime>,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

fn map_db(err: DatabaseError) -> StoreError {
    StoreError::Unexpected(err.to_string())
}

fn map_sqlx(err: sqlx::Error) -> StoreError {
    StoreError::Unexpected(err.to_string())
}

impl PostgresUserStore {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl PostgresRoomStore {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl PostgresSessionStore {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl mahfel_application::ports::UserStore for PostgresUserStore {
    async fn get(&self, id: UserId) -> Result<Option<User>, StoreError> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, display_name, status, created_at, updated_at \
             FROM mahfel.users WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx)?;

        match row {
            None => Ok(None),
            Some(r) => {
                let user = User::try_from(r).map_err(map_db)?;
                Ok(Some(user))
            }
        }
    }

    async fn insert(&self, user: &User) -> Result<(), StoreError> {
        sqlx::query(
            "INSERT INTO mahfel.users (id, display_name, status, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $4)",
        )
        .bind(user.id.as_uuid())
        .bind(user.display_name.as_str())
        .bind(user.status.as_str())
        .bind(user.created_at)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(())
    }

    async fn save(&self, user: &User) -> Result<(), StoreError> {
        let result = sqlx::query(
            "UPDATE mahfel.users \
             SET display_name = $2, status = $3, updated_at = $4 \
             WHERE id = $1",
        )
        .bind(user.id.as_uuid())
        .bind(user.display_name.as_str())
        .bind(user.status.as_str())
        .bind(user.updated_at)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;

        if result.rows_affected() == 0 {
            return Err(StoreError::NotFound(user.id.to_string()));
        }
        Ok(())
    }
}

impl mahfel_application::ports::RoomStore for PostgresRoomStore {
    async fn get(&self, id: RoomId) -> Result<Option<Room>, StoreError> {
        let row = sqlx::query_as::<_, RoomRow>(
            "SELECT id, title, created_by, status, created_at, updated_at \
             FROM mahfel.rooms WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx)?;

        match row {
            None => Ok(None),
            Some(r) => Ok(Some(Room::try_from(r).map_err(map_db)?)),
        }
    }

    async fn insert(&self, room: &Room) -> Result<(), StoreError> {
        sqlx::query(
            "INSERT INTO mahfel.rooms (id, title, created_by, status, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $5)",
        )
        .bind(room.id.as_uuid())
        .bind(room.title.as_str())
        .bind(room.created_by.as_uuid())
        .bind(room.status.as_str())
        .bind(room.created_at)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(())
    }

    async fn save(&self, room: &Room) -> Result<(), StoreError> {
        let result = sqlx::query(
            "UPDATE mahfel.rooms \
             SET title = $2, status = $3, updated_at = $4 \
             WHERE id = $1",
        )
        .bind(room.id.as_uuid())
        .bind(room.title.as_str())
        .bind(room.status.as_str())
        .bind(room.updated_at)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;

        if result.rows_affected() == 0 {
            return Err(StoreError::NotFound(room.id.to_string()));
        }
        Ok(())
    }
}

impl mahfel_application::ports::SessionStore for PostgresSessionStore {
    async fn get(&self, id: ClassroomSessionId) -> Result<Option<ClassroomSession>, StoreError> {
        let row = sqlx::query_as::<_, ClassroomSessionRow>(
            "SELECT id, room_id, status, scheduled_start_at, started_at, ended_at, \
                    created_at, updated_at \
             FROM mahfel.classroom_sessions WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx)?;

        match row {
            None => Ok(None),
            Some(r) => Ok(Some(ClassroomSession::try_from(r).map_err(map_db)?)),
        }
    }

    async fn insert(&self, session: &ClassroomSession) -> Result<(), StoreError> {
        sqlx::query(
            "INSERT INTO mahfel.classroom_sessions \
             (id, room_id, status, scheduled_start_at, started_at, ended_at, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $7)",
        )
        .bind(session.id.as_uuid())
        .bind(session.room_id.as_uuid())
        .bind(session.status.as_str())
        .bind(session.scheduled_start_at)
        .bind(session.started_at)
        .bind(session.ended_at)
        .bind(session.created_at)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(())
    }

    async fn save(&self, session: &ClassroomSession) -> Result<(), StoreError> {
        let result = sqlx::query(
            "UPDATE mahfel.classroom_sessions \
             SET status = $2, scheduled_start_at = $3, started_at = $4, ended_at = $5, updated_at = $6 \
             WHERE id = $1",
        )
        .bind(session.id.as_uuid())
        .bind(session.status.as_str())
        .bind(session.scheduled_start_at)
        .bind(session.started_at)
        .bind(session.ended_at)
        .bind(session.updated_at)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;

        if result.rows_affected() == 0 {
            return Err(StoreError::NotFound(session.id.to_string()));
        }
        Ok(())
    }
}

impl TryFrom<UserRow> for User {
    type Error = DatabaseError;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: UserId::from_uuid(row.id),
            display_name: DisplayName::new(row.display_name.clone()).map_err(|_| {
                DatabaseError::InvalidConfiguration("invalid display_name from database")
            })?,
            status: row.status.as_str().try_into()?,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<RoomRow> for Room {
    type Error = DatabaseError;

    fn try_from(row: RoomRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: RoomId::from_uuid(row.id),
            title: RoomTitle::new(row.title.clone()).map_err(|_| {
                DatabaseError::InvalidConfiguration("invalid room title from database")
            })?,
            created_by: UserId::from_uuid(row.created_by),
            status: row.status.as_str().try_into()?,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<ClassroomSessionRow> for ClassroomSession {
    type Error = DatabaseError;

    fn try_from(row: ClassroomSessionRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: ClassroomSessionId::from_uuid(row.id),
            room_id: RoomId::from_uuid(row.room_id),
            status: row.status.as_str().try_into()?,
            scheduled_start_at: row.scheduled_start_at,
            started_at: row.started_at,
            ended_at: row.ended_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
