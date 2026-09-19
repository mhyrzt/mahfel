use crate::DatabaseError;
use crate::settings::{ConnectionSettings, PoolSettings};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};

/// Open a connection pool for the configured database.
pub async fn connect(
    connection: &ConnectionSettings,
    settings: PoolSettings,
) -> Result<PgPool, DatabaseError> {
    let settings = settings.validate()?;
    let options = PgConnectOptions::new()
        .host(&connection.host)
        .port(connection.port)
        .username(&connection.user)
        .password(&connection.password)
        .database(&connection.name)
        // The database role and the `mahfel` schema share a name, and the
        // default search_path is `"$user", public`. Without pinning it,
        // unqualified `_sqlx_migrations` resolves to a different schema before
        // and after migration 0001 creates `mahfel`, splitting migration
        // bookkeeping. All application SQL qualifies `mahfel.` explicitly.
        .options([("search_path", "public")]);
    PgPoolOptions::new()
        .min_connections(settings.min_connections)
        .max_connections(settings.max_connections)
        .acquire_timeout(settings.acquire_timeout)
        .idle_timeout(Some(settings.idle_timeout))
        .max_lifetime(Some(settings.max_lifetime))
        .connect_with(options)
        .await
        .map_err(DatabaseError::from)
}

/// Cheap liveness probe for readiness reporting.
pub async fn ping(pool: &PgPool) -> Result<(), DatabaseError> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(DatabaseError::from)
}
