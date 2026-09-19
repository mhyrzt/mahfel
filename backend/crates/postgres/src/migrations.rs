use crate::DatabaseError;
use sqlx::{PgPool, migrate::Migrator};

static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

/// Apply pending forward migrations. Re-running on an up-to-date database is a
/// no-op.
pub async fn migrate(pool: &PgPool) -> Result<(), DatabaseError> {
    MIGRATOR.run(pool).await.map_err(DatabaseError::from)
}
