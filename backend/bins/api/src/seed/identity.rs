use anyhow::{Context, Result};
use mahfel_application::ports::IdentityProfile;
use mahfel_config::Settings;
use mahfel_domain::{ActorId, DisplayName, EmailAddress, Issuer, Subject, Username};
use mahfel_keycloak::KeycloakAdmin;
use mahfel_postgres::PostgresIdentityStore;
use serde::Deserialize;

use crate::seed::fixture::parse_actor_id;
use crate::settings;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DevelopmentUser {
    id: String,
    username: String,
    email: String,
    display_name: String,
    password: String,
}

pub(super) struct ValidatedPrincipal {
    id: ActorId,
    username: Username,
    email: EmailAddress,
    display_name: DisplayName,
}

pub(super) struct ValidatedUser {
    id: ActorId,
    username: Username,
    email: EmailAddress,
    display_name: DisplayName,
    password: String,
}

pub(super) fn validate_principal(
    id: String,
    username: String,
    email: String,
    display_name: String,
) -> Result<ValidatedPrincipal> {
    Ok(ValidatedPrincipal {
        id: parse_actor_id(&id)?,
        username: Username::new(username).context("validate principal username")?,
        email: EmailAddress::new(email).context("validate principal email")?,
        display_name: DisplayName::new(display_name).context("validate principal display name")?,
    })
}

pub(super) fn validate_users(users: Vec<DevelopmentUser>) -> Result<Vec<ValidatedUser>> {
    if users.is_empty() {
        anyhow::bail!("development users seed must contain at least one user");
    }
    users
        .into_iter()
        .map(|user| {
            if user.password.is_empty() {
                anyhow::bail!("development user password must not be empty");
            }
            Ok(ValidatedUser {
                id: parse_actor_id(&user.id)?,
                username: Username::new(user.username).context("validate development username")?,
                email: EmailAddress::new(user.email).context("validate development email")?,
                display_name: DisplayName::new(user.display_name)
                    .context("validate development display name")?,
                password: user.password,
            })
        })
        .collect()
}

pub(super) async fn apply_principal(
    identities: &PostgresIdentityStore,
    principal: &ValidatedPrincipal,
) -> Result<()> {
    identities
        .seed_development_principal(
            principal.id,
            &principal.username,
            &principal.email,
            &principal.display_name,
            time::OffsetDateTime::now_utc(),
        )
        .await?;
    Ok(())
}

pub(super) async fn apply_users(
    identities: &PostgresIdentityStore,
    settings: &Settings,
    issuer: &Issuer,
    users: &[ValidatedUser],
) -> Result<()> {
    let admin = KeycloakAdmin::new(settings::keycloak(settings));
    for user in users {
        let (subject, created) = admin
            .create_user_if_missing(
                user.username.as_str(),
                user.email.as_str(),
                user.display_name.as_str(),
                &user.password,
            )
            .await
            .with_context(|| format!("ensure Keycloak user {}", user.username.as_str()))?;
        if created {
            admin
                .set_user_attribute(&subject, "mahfel_user_id", &user.id.as_uuid().to_string())
                .await
                .with_context(|| format!("link new Keycloak user {}", user.username.as_str()))?;
        }
        let subject = Subject::new(subject).context("validate Keycloak user subject")?;
        identities
            .mirror_development_identity(
                user.id,
                issuer,
                &subject,
                &user.display_name,
                &IdentityProfile {
                    display_name: Some(user.display_name.clone()),
                    email: Some(user.email.clone()),
                    username: user.username.clone(),
                    email_verified: true,
                    avatar_url: None,
                },
                time::OffsetDateTime::now_utc(),
            )
            .await
            .with_context(|| format!("mirror development user {}", user.username.as_str()))?;
        tracing::info!(username = %user.username.as_str(), "development identity ensured");
    }
    Ok(())
}
