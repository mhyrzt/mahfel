use crate::DatabaseError;
use crate::stores::store_error;
use mahfel_application::ports::{BoxFuture, ServicePrincipalStore, StoreError};
use mahfel_domain::{ActorId, DisplayName, ServicePrincipal, ServicePrincipalStatus};
use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresServicePrincipalStore {
    pool: PgPool,
}

impl PostgresServicePrincipalStore {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct ServicePrincipalRow {
    id: Uuid,
    name: String,
    display_name: Option<String>,
    status: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

impl TryFrom<ServicePrincipalRow> for ServicePrincipal {
    type Error = DatabaseError;

    fn try_from(row: ServicePrincipalRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: ActorId::from_uuid(row.id),
            name: DisplayName::new(row.name)
                .map_err(|_| DatabaseError::InvalidStoredValue("invalid service principal name"))?,
            display_name: row
                .display_name
                .map(|value| {
                    DisplayName::new(value).map_err(|_| {
                        DatabaseError::InvalidStoredValue("invalid service principal display name")
                    })
                })
                .transpose()?,
            status: row.status.as_str().try_into()?,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

const SELECT_PRINCIPAL_BY_ID: &str = "SELECT id, name, display_name, status, created_at, updated_at \
     FROM mahfel.service_principals WHERE id = $1";

const SELECT_PRINCIPALS: &str = "SELECT id, name, display_name, status, created_at, updated_at \
     FROM mahfel.service_principals ORDER BY created_at DESC, id";

/// Map write-time constraints to typed application errors.
///
/// A unique violation (`23505`) is a name conflict; a foreign-key violation
/// (`23503`) means a referenced actor row is missing.
fn sqlx_write_error(err: sqlx::Error) -> StoreError {
    if let sqlx::Error::Database(database) = &err {
        match database.code().as_deref() {
            Some("23505") => return StoreError::Conflict(err.to_string()),
            Some("23503") => return StoreError::MissingReference(err.to_string()),
            _ => {}
        }
    }
    StoreError::Unexpected(err.to_string())
}

const REVOKE_ACTIVE_KEYS: &str = "UPDATE mahfel.service_api_keys \
     SET status = 'revoked', updated_at = $2, revoked_at = $2 \
     WHERE service_principal_id = $1 AND status = 'active'";

impl ServicePrincipalStore for PostgresServicePrincipalStore {
    fn create<'a>(
        &'a self,
        principal: &'a ServicePrincipal,
        api_key_hash: &'a str,
    ) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let mut transaction = self.pool.begin().await.map_err(store_error)?;

            sqlx::query(
                "INSERT INTO mahfel.actors (id, kind, created_at, updated_at) \
                 VALUES ($1, 'service', $2, $2)",
            )
            .bind(principal.id.as_uuid())
            .bind(principal.created_at)
            .execute(&mut *transaction)
            .await
            .map_err(sqlx_write_error)?;

            sqlx::query(
                "INSERT INTO mahfel.service_principals \
                 (id, name, display_name, status, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, $5)",
            )
            .bind(principal.id.as_uuid())
            .bind(principal.name.as_str())
            .bind(principal.display_name.as_ref().map(DisplayName::as_str))
            .bind(principal.status.as_str())
            .bind(principal.created_at)
            .execute(&mut *transaction)
            .await
            .map_err(sqlx_write_error)?;

            sqlx::query(
                "INSERT INTO mahfel.service_api_keys \
                 (id, service_principal_id, key_hash, status, created_at, updated_at) \
                 VALUES ($1, $2, $3, 'active', $4, $4)",
            )
            .bind(Uuid::new_v4())
            .bind(principal.id.as_uuid())
            .bind(api_key_hash)
            .bind(principal.created_at)
            .execute(&mut *transaction)
            .await
            .map_err(sqlx_write_error)?;

            transaction.commit().await.map_err(store_error)?;
            Ok(())
        })
    }

    fn get(&self, id: ActorId) -> BoxFuture<'_, Result<Option<ServicePrincipal>, StoreError>> {
        Box::pin(async move {
            let row = sqlx::query_as::<_, ServicePrincipalRow>(SELECT_PRINCIPAL_BY_ID)
                .bind(id.as_uuid())
                .fetch_optional(&self.pool)
                .await
                .map_err(store_error)?;
            row.map(|row| ServicePrincipal::try_from(row).map_err(store_error))
                .transpose()
        })
    }

    fn list(&self) -> BoxFuture<'_, Result<Vec<ServicePrincipal>, StoreError>> {
        Box::pin(async move {
            let rows = sqlx::query_as::<_, ServicePrincipalRow>(SELECT_PRINCIPALS)
                .fetch_all(&self.pool)
                .await
                .map_err(store_error)?;
            rows.into_iter()
                .map(|row| ServicePrincipal::try_from(row).map_err(store_error))
                .collect()
        })
    }

    fn revoke(&self, id: ActorId) -> BoxFuture<'_, Result<(), StoreError>> {
        Box::pin(async move {
            let now = OffsetDateTime::now_utc();
            let mut transaction = self.pool.begin().await.map_err(store_error)?;

            let updated = sqlx::query(
                "UPDATE mahfel.service_principals \
                 SET status = 'revoked', updated_at = $2 WHERE id = $1",
            )
            .bind(id.as_uuid())
            .bind(now)
            .execute(&mut *transaction)
            .await
            .map_err(store_error)?;
            if updated.rows_affected() == 0 {
                return Err(StoreError::NotFound(id.to_string()));
            }

            sqlx::query(REVOKE_ACTIVE_KEYS)
                .bind(id.as_uuid())
                .bind(now)
                .execute(&mut *transaction)
                .await
                .map_err(store_error)?;

            transaction.commit().await.map_err(store_error)?;
            Ok(())
        })
    }

    fn rotate_key<'a>(
        &'a self,
        id: ActorId,
        api_key_hash: &'a str,
    ) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let now = OffsetDateTime::now_utc();
            let mut transaction = self.pool.begin().await.map_err(store_error)?;

            // Lock the principal and require it to be active, so rotation cannot
            // revive a revoked principal or race a concurrent revocation.
            let status = sqlx::query_scalar::<_, String>(
                "SELECT status FROM mahfel.service_principals WHERE id = $1 FOR UPDATE",
            )
            .bind(id.as_uuid())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(store_error)?;
            if status.as_deref() != Some(ServicePrincipalStatus::Active.as_str()) {
                return Err(StoreError::NotFound(id.to_string()));
            }

            sqlx::query(REVOKE_ACTIVE_KEYS)
                .bind(id.as_uuid())
                .bind(now)
                .execute(&mut *transaction)
                .await
                .map_err(store_error)?;

            sqlx::query(
                "INSERT INTO mahfel.service_api_keys \
                 (id, service_principal_id, key_hash, status, created_at, updated_at) \
                 VALUES ($1, $2, $3, 'active', $4, $4)",
            )
            .bind(Uuid::new_v4())
            .bind(id.as_uuid())
            .bind(api_key_hash)
            .bind(now)
            .execute(&mut *transaction)
            .await
            .map_err(sqlx_write_error)?;

            transaction.commit().await.map_err(store_error)?;
            Ok(())
        })
    }

    fn find_active_by_key_hash<'a>(
        &'a self,
        api_key_hash: &'a str,
    ) -> BoxFuture<'a, Result<Option<ServicePrincipal>, StoreError>> {
        Box::pin(async move {
            let row = sqlx::query_as::<_, ServicePrincipalRow>(
                "SELECT sp.id, sp.name, sp.display_name, sp.status, sp.created_at, sp.updated_at \
                 FROM mahfel.service_api_keys k \
                 JOIN mahfel.service_principals sp ON sp.id = k.service_principal_id \
                 WHERE k.key_hash = $1 AND k.status = 'active' AND sp.status = 'active'",
            )
            .bind(api_key_hash)
            .fetch_optional(&self.pool)
            .await
            .map_err(store_error)?;

            let Some(row) = row else {
                return Ok(None);
            };

            // Best-effort usage tracking: a failed touch must not fail authentication.
            let _ = sqlx::query(
                "UPDATE mahfel.service_api_keys SET last_used_at = $2 \
                 WHERE key_hash = $1 AND status = 'active'",
            )
            .bind(api_key_hash)
            .bind(OffsetDateTime::now_utc())
            .execute(&self.pool)
            .await;

            Ok(Some(ServicePrincipal::try_from(row).map_err(store_error)?))
        })
    }
}
