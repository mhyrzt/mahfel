mod assets;
mod files;
mod fixture;
mod identity;
mod rooms;

use std::path::PathBuf;

use anyhow::{Context, Result};
use mahfel_config::Settings;
use mahfel_domain::Issuer;
use mahfel_postgres::{
    PostgresFileStore, PostgresIdentityStore, PostgresRoomStore, PostgresSessionStore, connect,
};
use mahfel_storage::{S3FileStorage, S3Settings};

use crate::settings;
use fixture::Fixture;

/// Apply typed development fixtures explicitly, in file-name order.
/// All selected files are parsed and validated before the first external write.
pub(crate) async fn run(settings: Settings, path: Option<PathBuf>) -> Result<()> {
    let sources = match path {
        Some(path) => files::read_path(&path)?,
        None => files::embedded_seeds(),
    };
    let mut seeds = fixture::parse_seeds(sources)?;
    assets::preflight(&mut seeds).await?;
    let issuer = if seeds.iter().any(|(_, fixture)| fixture.needs_issuer()) {
        Some(Issuer::new(settings.oidc.issuer_url.clone()).context("validate Keycloak issuer")?)
    } else {
        None
    };
    let pool = connect(&settings::connection(&settings), settings::pool(&settings))
        .await
        .context("connect to PostgreSQL for seeding")?;
    let identities = PostgresIdentityStore::new(pool.clone());
    let room_store = PostgresRoomStore::new(pool.clone());
    let session_store = PostgresSessionStore::new(pool.clone());
    let file_store = PostgresFileStore::new(pool.clone());
    let storage = S3FileStorage::new(S3Settings::from(&settings.storage));

    for (name, fixture) in &seeds {
        match fixture {
            Fixture::DevelopmentPrincipal(principal) => {
                identity::apply_principal(&identities, principal)
                    .await
                    .with_context(|| format!("apply seed {name}"))?;
            }
            Fixture::DevelopmentUsers(users) => {
                let issuer = issuer
                    .as_ref()
                    .context("development users require an issuer")?;
                identity::apply_users(&identities, &settings, issuer, users).await?;
            }
            Fixture::Rooms(fixtures) => {
                rooms::apply_rooms(&room_store, fixtures, name).await?;
            }
            Fixture::Sessions(fixtures) => {
                rooms::apply_sessions(&session_store, fixtures, name).await?;
            }
            Fixture::Files(fixtures) => {
                assets::apply_assets(&file_store, &room_store, &storage, fixtures, name).await?;
            }
        }
        tracing::info!(seed = %name, "seed applied");
    }
    pool.close().await;
    tracing::info!(count = seeds.len(), "development seeds applied");
    Ok(())
}
