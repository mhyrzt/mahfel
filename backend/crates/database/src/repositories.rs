use crate::DatabaseError;
use mahfel_domain::{
    ClassroomSession, ClassroomSessionId, Room, RoomId, RoomStatus, SessionStatus, User, UserId,
    UserStatus,
};
use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

#[derive(Clone)]
pub struct RoomRepository {
    pool: PgPool,
}

#[derive(Clone)]
pub struct ClassroomSessionRepository {
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

impl UserRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, display_name: &str) -> Result<User, DatabaseError> {
        let id = UserId::new();
        let now = OffsetDateTime::now_utc();
        let row = sqlx::query_as::<_, UserRow>(
            "INSERT INTO mahfel.users (id, display_name, status, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $4) \
             RETURNING id, display_name, status, created_at, updated_at",
        )
        .bind(id.as_uuid())
        .bind(display_name)
        .bind(UserStatus::Active.as_str())
        .bind(now)
        .fetch_one(&self.pool)
        .await?;
        row.try_into()
    }

    pub async fn get(&self, id: UserId) -> Result<Option<User>, DatabaseError> {
        sqlx::query_as::<_, UserRow>(
            "SELECT id, display_name, status, created_at, updated_at \
             FROM mahfel.users WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await?
        .map(TryInto::try_into)
        .transpose()
    }
}

impl RoomRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, title: &str, created_by: UserId) -> Result<Room, DatabaseError> {
        let id = RoomId::new();
        let now = OffsetDateTime::now_utc();
        let row = sqlx::query_as::<_, RoomRow>(
            "INSERT INTO mahfel.rooms (id, title, created_by, status, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $5) \
             RETURNING id, title, created_by, status, created_at, updated_at",
        )
        .bind(id.as_uuid())
        .bind(title)
        .bind(created_by.as_uuid())
        .bind(RoomStatus::Active.as_str())
        .bind(now)
        .fetch_one(&self.pool)
        .await?;
        row.try_into()
    }

    pub async fn get(&self, id: RoomId) -> Result<Option<Room>, DatabaseError> {
        sqlx::query_as::<_, RoomRow>(
            "SELECT id, title, created_by, status, created_at, updated_at \
             FROM mahfel.rooms WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await?
        .map(TryInto::try_into)
        .transpose()
    }
}

impl ClassroomSessionRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        room_id: RoomId,
        scheduled_start_at: Option<OffsetDateTime>,
    ) -> Result<ClassroomSession, DatabaseError> {
        let id = ClassroomSessionId::new();
        let now = OffsetDateTime::now_utc();
        let row = sqlx::query_as::<_, ClassroomSessionRow>(
            "INSERT INTO mahfel.classroom_sessions \
             (id, room_id, status, scheduled_start_at, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $5) \
             RETURNING id, room_id, status, scheduled_start_at, started_at, ended_at, \
                       created_at, updated_at",
        )
        .bind(id.as_uuid())
        .bind(room_id.as_uuid())
        .bind(SessionStatus::Scheduled.as_str())
        .bind(scheduled_start_at)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;
        row.try_into()
    }

    pub async fn get(
        &self,
        id: ClassroomSessionId,
    ) -> Result<Option<ClassroomSession>, DatabaseError> {
        sqlx::query_as::<_, ClassroomSessionRow>(
            "SELECT id, room_id, status, scheduled_start_at, started_at, ended_at, \
                    created_at, updated_at \
             FROM mahfel.classroom_sessions WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await?
        .map(TryInto::try_into)
        .transpose()
    }
}

impl TryFrom<UserRow> for User {
    type Error = DatabaseError;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: UserId::from_uuid(row.id),
            display_name: row.display_name,
            status: UserStatus::try_from(row.status.as_str())?,
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
            title: row.title,
            created_by: UserId::from_uuid(row.created_by),
            status: RoomStatus::try_from(row.status.as_str())?,
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
            status: SessionStatus::try_from(row.status.as_str())?,
            scheduled_start_at: row.scheduled_start_at,
            started_at: row.started_at,
            ended_at: row.ended_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
