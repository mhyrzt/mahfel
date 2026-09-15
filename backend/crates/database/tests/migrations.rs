use mahfel_database::{PoolSettings, connect, migrate};
use sqlx::PgPool;
use std::{error::Error, time::Duration};
use url::Url;
use uuid::Uuid;

fn test_pool_settings() -> PoolSettings {
    PoolSettings {
        min_connections: 1,
        max_connections: 2,
        acquire_timeout: Duration::from_secs(5),
        idle_timeout: Duration::from_secs(30),
        max_lifetime: Duration::from_secs(60),
    }
}

#[tokio::test]
async fn migrations_provision_a_disposable_database() -> Result<(), Box<dyn Error>> {
    let Ok(admin_url) = std::env::var("MAHFEL_TEST_DATABASE_URL") else {
        eprintln!("MAHFEL_TEST_DATABASE_URL is unset; skipping PostgreSQL integration test");
        return Ok(());
    };

    let admin_pool = PgPool::connect(&admin_url).await?;
    let database_name = format!("mahfel_test_{}", Uuid::new_v4().simple());
    let create_database = format!("CREATE DATABASE \"{database_name}\"");
    // The interpolated identifier is generated exclusively from a UUID.
    sqlx::raw_sql(sqlx::AssertSqlSafe(create_database))
        .execute(&admin_pool)
        .await?;

    let mut test_url = Url::parse(&admin_url)?;
    test_url.set_path(&format!("/{database_name}"));
    let test_pool = connect(test_url.as_str(), test_pool_settings()).await?;
    let verification = verify_migrations(&test_pool).await;
    test_pool.close().await;

    let drop_database = format!("DROP DATABASE \"{database_name}\" WITH (FORCE)");
    let cleanup = sqlx::raw_sql(sqlx::AssertSqlSafe(drop_database))
        .execute(&admin_pool)
        .await;
    admin_pool.close().await;

    verification?;
    cleanup?;
    Ok(())
}

async fn verify_migrations(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    migrate(pool).await?;
    migrate(pool).await?;

    let migration_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(pool)
        .await?;
    assert_eq!(migration_count, 1, "migration must be recorded once");

    let namespace_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'mahfel')",
    )
    .fetch_one(pool)
    .await?;
    assert!(
        namespace_exists,
        "migration must provision the Mahfel schema"
    );
    Ok(())
}
