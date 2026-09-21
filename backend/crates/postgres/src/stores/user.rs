use crate::DatabaseError;
use crate::stores::store_error;
use mahfel_application::ports::{BoxFuture, StoreError, UserStore};
use mahfel_domain::{ActorId, AvatarUrl, DisplayName, User};
use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
pub(crate) struct UserRow {
    id: Uuid,
    display_name: String,
    status: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    avatar_url: Option<String>,
}

impl UserStore for PostgresUserStore {
    fn get(&self, id: ActorId) -> BoxFuture<'_, Result<Option<User>, StoreError>> {
        Box::pin(async move {
            let row = sqlx::query_as::<_, UserRow>(
                "SELECT id, display_name, status, created_at, updated_at, avatar_url \
                 FROM mahfel.users WHERE id = $1",
            )
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(store_error)?;

            row.map(|row| User::try_from(row).map_err(store_error))
                .transpose()
        })
    }

    fn insert<'a>(&'a self, user: &'a User) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            // A user and its actor row share an id and must be created together so
            // no user can exist without a matching actor.
            let mut transaction = self.pool.begin().await.map_err(store_error)?;
            sqlx::query(
                "INSERT INTO mahfel.actors (id, kind, created_at, updated_at) \
                 VALUES ($1, 'user', $2, $2)",
            )
            .bind(user.id.as_uuid())
            .bind(user.created_at)
            .execute(&mut *transaction)
            .await
            .map_err(store_error)?;
            sqlx::query(
                "INSERT INTO mahfel.users (id, display_name, status, created_at, updated_at, avatar_url) \
                 VALUES ($1, $2, $3, $4, $4, $5)",
            )
            .bind(user.id.as_uuid())
            .bind(user.display_name.as_str())
            .bind(user.status.as_str())
            .bind(user.created_at)
            .bind(user.avatar_url.as_ref().map(AvatarUrl::as_str))
            .execute(&mut *transaction)
            .await
            .map_err(store_error)?;
            transaction.commit().await.map_err(store_error)?;
            Ok(())
        })
    }

    fn save<'a>(&'a self, user: &'a User) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let result = sqlx::query(
                "UPDATE mahfel.users \
                 SET display_name = $2, status = $3, updated_at = $4, avatar_url = $5 \
                 WHERE id = $1",
            )
            .bind(user.id.as_uuid())
            .bind(user.display_name.as_str())
            .bind(user.status.as_str())
            .bind(user.updated_at)
            .bind(user.avatar_url.as_ref().map(AvatarUrl::as_str))
            .execute(&self.pool)
            .await
            .map_err(store_error)?;

            if result.rows_affected() == 0 {
                return Err(StoreError::NotFound(user.id.to_string()));
            }
            Ok(())
        })
    }
}

impl TryFrom<UserRow> for User {
    type Error = DatabaseError;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        let avatar_url = row
            .avatar_url
            .map(|value| {
                AvatarUrl::new(value)
                    .map_err(|_| DatabaseError::InvalidStoredValue("invalid avatar url"))
            })
            .transpose()?;
        Ok(Self {
            id: ActorId::from_uuid(row.id),
            display_name: DisplayName::new(row.display_name.clone())
                .map_err(|_| DatabaseError::InvalidStoredValue("invalid display name"))?,
            status: row.status.as_str().try_into()?,
            avatar_url,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
