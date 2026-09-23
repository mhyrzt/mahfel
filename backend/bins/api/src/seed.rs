use crate::settings;
use anyhow::{Context, Result};
use mahfel_application::ports::IdentityProfile;
use mahfel_config::Settings;
use mahfel_domain::{ActorId, DisplayName, EmailAddress, Issuer, Subject, Username};
use mahfel_keycloak::KeycloakAdmin;
use mahfel_postgres::{PostgresIdentityStore, connect};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/seeds.rs"));
}

#[derive(Deserialize)]
struct DevelopmentUser {
    id: String,
    username: String,
    email: String,
    display_name: String,
    password: String,
}

const DEVELOPMENT_USERS: &str = include_str!("../../../seeds/development-users.json");

/// Apply development seed fixtures explicitly, in file-name order.
///
/// Without a path, every seed embedded at build time (see `build.rs`) is
/// applied. An explicit file or directory is read from disk instead, which
/// makes applying a single fixture quick during development. Seeds are never
/// applied by migrations or normal startup, and fixtures must be idempotent so
/// the command can be re-run safely.
pub(crate) async fn run(settings: Settings, path: Option<PathBuf>) -> Result<()> {
    let seed_development_users = path.is_none();
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

    if seed_development_users {
        let admin = KeycloakAdmin::new(crate::settings::keycloak(&settings));
        let issuer = Issuer::new(settings.oidc.issuer_url.clone())
            .context("validate Keycloak issuer for development seed")?;
        let identities = PostgresIdentityStore::new(pool.clone());
        let users: Vec<DevelopmentUser> =
            serde_json::from_str(DEVELOPMENT_USERS).context("decode embedded development users")?;

        for fixture in users {
            let (subject, created) = admin
                .create_user_if_missing(
                    &fixture.username,
                    &fixture.email,
                    &fixture.display_name,
                    &fixture.password,
                )
                .await
                .with_context(|| format!("ensure Keycloak user {}", fixture.username))?;
            if created {
                admin
                    .set_user_attribute(&subject, "mahfel_user_id", &fixture.id)
                    .await
                    .with_context(|| format!("link new Keycloak user {}", fixture.username))?;
            }
            let subject = Subject::new(subject).context("validate Keycloak user subject")?;
            let user_id = ActorId::from_uuid(
                uuid::Uuid::parse_str(&fixture.id).context("validate fixture user id")?,
            );
            let display_name = DisplayName::new(fixture.display_name)
                .context("validate development display name")?;
            let email = EmailAddress::new(fixture.email).context("validate development email")?;
            let fixture_username = fixture.username.clone();
            let username =
                Username::new(fixture_username.clone()).context("validate development username")?;
            identities
                .mirror_development_identity(
                    user_id,
                    &issuer,
                    &subject,
                    &display_name,
                    &IdentityProfile {
                        display_name: Some(display_name.clone()),
                        email: Some(email),
                        username,
                        email_verified: true,
                        avatar_url: None,
                    },
                    time::OffsetDateTime::now_utc(),
                )
                .await
                .with_context(|| format!("mirror development user {fixture_username}"))?;
            tracing::info!(username = %fixture.username, "development identity ensured");
        }
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
