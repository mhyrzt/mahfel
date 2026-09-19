use crate::settings;
use anyhow::{Context, Result};
use mahfel_config::Settings;
use mahfel_postgres::{connect, migrate};

pub(crate) async fn run(settings: Settings) -> Result<()> {
    let pool = connect(&settings::connection(&settings), settings::pool(&settings))
        .await
        .context("connect to PostgreSQL for migration")?;
    migrate(&pool).await.context("run PostgreSQL migrations")?;
    pool.close().await;
    tracing::info!("database migrations completed");
    Ok(())
}
