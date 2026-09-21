use crate::stores::sqlx_store_error;
use mahfel_application::ports::{BoxFuture, InviteStore, StoreError};
use mahfel_domain::{
    ActorId, InviteCode, InviteLink, InviteLinkId, InviteLinkStatus, InviteTarget,
    InviteTargetKind, RoomId, SessionId,
};
use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresInviteStore {
    pool: PgPool,
}

impl PostgresInviteStore {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct InviteRow {
    id: Uuid,
    code: String,
    target_kind: String,
    target_id: Uuid,
    status: String,
    expires_at: OffsetDateTime,
    created_by: Uuid,
    created_at: OffsetDateTime,
    revoked_at: Option<OffsetDateTime>,
}

impl InviteRow {
    fn into_link(self) -> Result<InviteLink, StoreError> {
        let code = InviteCode::parse(&self.code)
            .map_err(|error| StoreError::Unexpected(error.to_string()))?;
        let target_kind: InviteTargetKind = self.target_kind.as_str().try_into().map_err(
            |error: mahfel_domain::InvalidStatus| StoreError::Unexpected(error.to_string()),
        )?;
        let target = match target_kind {
            InviteTargetKind::Room => InviteTarget::Room(RoomId::from_uuid(self.target_id)),
            InviteTargetKind::Session => {
                InviteTarget::Session(SessionId::from_uuid(self.target_id))
            }
        };
        let status: InviteLinkStatus =
            self.status
                .as_str()
                .try_into()
                .map_err(|error: mahfel_domain::InvalidStatus| {
                    StoreError::Unexpected(error.to_string())
                })?;
        Ok(InviteLink {
            id: InviteLinkId::from_uuid(self.id),
            code,
            target,
            status,
            expires_at: self.expires_at,
            created_by: ActorId::from_uuid(self.created_by),
            created_at: self.created_at,
            revoked_at: self.revoked_at,
        })
    }
}

const SELECT_COLUMNS: &str = "id, code, target_kind, target_id, status, expires_at, \
                              created_by, created_at, revoked_at";

impl InviteStore for PostgresInviteStore {
    fn insert<'a>(&'a self, link: &'a InviteLink) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let (target_kind, target_id) = split_target(link.target);
            sqlx::query(
                "INSERT INTO mahfel.invite_links \
                 (id, code, target_kind, target_id, status, expires_at, created_by, created_at, \
                  revoked_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            )
            .bind(link.id.as_uuid())
            .bind(link.code.as_str())
            .bind(target_kind)
            .bind(target_id)
            .bind(link.status.as_str())
            .bind(link.expires_at)
            .bind(link.created_by.as_uuid())
            .bind(link.created_at)
            .bind(link.revoked_at)
            .execute(&self.pool)
            .await
            .map_err(sqlx_store_error)?;
            Ok(())
        })
    }

    fn get_by_code(
        &self,
        code: InviteCode,
    ) -> BoxFuture<'_, Result<Option<InviteLink>, StoreError>> {
        Box::pin(async move {
            let query = format!("SELECT {SELECT_COLUMNS} FROM mahfel.invite_links WHERE code = $1");
            let row = sqlx::query_as::<_, InviteRow>(sqlx::AssertSqlSafe(query))
                .bind(code.as_str())
                .fetch_optional(&self.pool)
                .await
                .map_err(sqlx_store_error)?;
            row.map(InviteRow::into_link).transpose()
        })
    }

    fn get(&self, id: InviteLinkId) -> BoxFuture<'_, Result<Option<InviteLink>, StoreError>> {
        Box::pin(async move {
            let query = format!("SELECT {SELECT_COLUMNS} FROM mahfel.invite_links WHERE id = $1");
            let row = sqlx::query_as::<_, InviteRow>(sqlx::AssertSqlSafe(query))
                .bind(id.as_uuid())
                .fetch_optional(&self.pool)
                .await
                .map_err(sqlx_store_error)?;
            row.map(InviteRow::into_link).transpose()
        })
    }

    fn list_for_target(
        &self,
        target_kind: InviteTargetKind,
        target_id: Uuid,
        limit: u32,
    ) -> BoxFuture<'_, Result<Vec<InviteLink>, StoreError>> {
        Box::pin(async move {
            let query = format!(
                "SELECT {SELECT_COLUMNS} FROM mahfel.invite_links \
                 WHERE target_kind = $1 AND target_id = $2 \
                 ORDER BY created_at DESC, id LIMIT $3"
            );
            let rows = sqlx::query_as::<_, InviteRow>(sqlx::AssertSqlSafe(query))
                .bind(target_kind.as_str())
                .bind(target_id)
                .bind(i64::from(limit))
                .fetch_all(&self.pool)
                .await
                .map_err(sqlx_store_error)?;
            rows.into_iter().map(InviteRow::into_link).collect()
        })
    }

    fn revoke_if_active(
        &self,
        id: InviteLinkId,
        revoked_at: OffsetDateTime,
    ) -> BoxFuture<'_, Result<bool, StoreError>> {
        Box::pin(async move {
            // Conditional so two concurrent revokes cannot both report success.
            let result = sqlx::query(
                "UPDATE mahfel.invite_links \
                 SET status = 'revoked', revoked_at = $2 \
                 WHERE id = $1 AND status <> 'revoked'",
            )
            .bind(id.as_uuid())
            .bind(revoked_at)
            .execute(&self.pool)
            .await
            .map_err(sqlx_store_error)?;
            Ok(result.rows_affected() == 1)
        })
    }

    fn rotate<'a>(
        &'a self,
        previous: InviteLinkId,
        revoked_at: OffsetDateTime,
        replacement: &'a InviteLink,
    ) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let (target_kind, target_id) = split_target(replacement.target);
            let mut tx = self.pool.begin().await.map_err(sqlx_store_error)?;

            // Revoke and insert commit together so a reader sees exactly one
            // live code.
            let revoked = sqlx::query(
                "UPDATE mahfel.invite_links \
                 SET status = 'revoked', revoked_at = $2 \
                 WHERE id = $1 AND status <> 'revoked'",
            )
            .bind(previous.as_uuid())
            .bind(revoked_at)
            .execute(&mut *tx)
            .await
            .map_err(sqlx_store_error)?;
            if revoked.rows_affected() == 0 {
                return Err(StoreError::Conflict(
                    "invite link is already revoked".to_owned(),
                ));
            }

            // A duplicate code is the caller's retry signal; dropping `tx`
            // without committing leaves the previous code live.
            sqlx::query(
                "INSERT INTO mahfel.invite_links \
                 (id, code, target_kind, target_id, status, expires_at, created_by, created_at, \
                  revoked_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            )
            .bind(replacement.id.as_uuid())
            .bind(replacement.code.as_str())
            .bind(target_kind)
            .bind(target_id)
            .bind(replacement.status.as_str())
            .bind(replacement.expires_at)
            .bind(replacement.created_by.as_uuid())
            .bind(replacement.created_at)
            .bind(replacement.revoked_at)
            .execute(&mut *tx)
            .await
            .map_err(sqlx_store_error)?;

            tx.commit().await.map_err(sqlx_store_error)?;
            Ok(())
        })
    }
}

fn split_target(target: InviteTarget) -> (&'static str, Uuid) {
    (target.kind().as_str(), target.id())
}
