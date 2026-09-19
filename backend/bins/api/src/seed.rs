use crate::settings;
use anyhow::{Context, Result};
use mahfel_config::Settings;
use mahfel_postgres::connect;
use std::{
    fs,
    path::{Path, PathBuf},
};

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/seeds.rs"));
}

/// Apply development seed fixtures explicitly, in file-name order.
///
/// Without a path, every seed embedded at build time (see `build.rs`) is
/// applied. An explicit file or directory is read from disk instead, which
/// makes applying a single fixture quick during development. Seeds are never
/// applied by migrations or normal startup, and fixtures must be idempotent so
/// the command can be re-run safely.
pub(crate) async fn run(settings: Settings, path: Option<PathBuf>) -> Result<()> {
    let seeds = match path {
        Some(path) => read_path(&path)?,
        None => embedded_seeds(),
    };
    let pool = connect(&settings::connection(&settings), settings::pool(&settings))
        .await
        .context("connect to PostgreSQL for seeding")?;

    for (name, statement) in &seeds {
        sqlx::raw_sql(sqlx::AssertSqlSafe(statement.as_str()))
            .execute(&pool)
            .await
            .with_context(|| format!("apply seed {name}"))?;
        tracing::info!(seed = %name, "seed applied");
    }

    pool.close().await;
    tracing::info!(count = seeds.len(), "development seeds applied");
    Ok(())
}

fn embedded_seeds() -> Vec<(String, String)> {
    embedded::SEEDS
        .iter()
        .map(|(name, statement)| ((*name).to_owned(), (*statement).to_owned()))
        .collect()
}

fn read_path(path: &Path) -> Result<Vec<(String, String)>> {
    if path.is_dir() {
        let mut files = seed_files(path)?;
        if files.is_empty() {
            anyhow::bail!("no .sql seeds found in {}", path.display());
        }
        files.drain(..).map(|file| read_file(&file)).collect()
    } else if path.is_file() {
        Ok(vec![read_file(path)?])
    } else {
        anyhow::bail!("seed path does not exist: {}", path.display())
    }
}

fn seed_files(directory: &Path) -> Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = fs::read_dir(directory)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "sql"))
        .collect();
    files.sort();
    Ok(files)
}

fn read_file(path: &Path) -> Result<(String, String)> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("seed")
        .to_owned();
    let statement =
        fs::read_to_string(path).with_context(|| format!("read seed {}", path.display()))?;
    Ok((name, statement))
}
