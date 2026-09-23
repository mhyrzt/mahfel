//! Keycloak Admin API adapter (ADR 0019).
//!
//! The API links a local user to its Keycloak identity by writing a user
//! attribute that a realm protocol mapper exposes as the `app_user_id` claim.
//! Authentication is the client-credentials grant of a dedicated service
//! account; the adapter never uses an interactive administrator account.

use crate::{error::KeycloakError, settings::KeycloakAdminSettings};
use serde::Deserialize;
use serde_json::json;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

/// Refresh the cached admin token this long before it actually expires.
const TOKEN_REFRESH_SKEW: Duration = Duration::from_secs(10);

/// Client-credentials administration of one Keycloak realm.
#[derive(Clone)]
pub struct KeycloakAdmin {
    inner: Arc<Inner>,
}

struct Inner {
    settings: KeycloakAdminSettings,
    http: reqwest::Client,
    token: Mutex<Option<CachedToken>>,
}

struct CachedToken {
    value: String,
    expires_at: Instant,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: Option<u64>,
}

impl KeycloakAdmin {
    #[must_use]
    pub fn new(settings: KeycloakAdminSettings) -> Self {
        let http = reqwest::Client::builder()
            .timeout(settings.http_timeout)
            .build()
            .unwrap_or_default();
        Self {
            inner: Arc::new(Inner {
                settings,
                http,
                token: Mutex::new(None),
            }),
        }
    }

    /// Set (or replace) a user attribute idempotently.
    pub async fn set_user_attribute(
        &self,
        user_id: &str,
        name: &str,
        value: &str,
    ) -> Result<(), KeycloakError> {
        let token = self.access_token().await?;
        let url = format!(
            "{}/admin/realms/{}/users/{}",
            self.inner.settings.base_url.trim_end_matches('/'),
            self.inner.settings.realm,
            user_id
        );
        let mut body: serde_json::Value = self
            .inner
            .http
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(transport)?
            .error_for_status()
            .map_err(|error| {
                status_error(
                    error
                        .status()
                        .unwrap_or(reqwest::StatusCode::INTERNAL_SERVER_ERROR),
                )
            })?
            .json()
            .await
            .map_err(|error| {
                KeycloakError::Unexpected(format!("decode Keycloak user representation: {error}"))
            })?;
        let attributes = body
            .as_object_mut()
            .ok_or_else(|| {
                KeycloakError::Unexpected("invalid Keycloak user representation".to_owned())
            })?
            .entry("attributes")
            .or_insert_with(|| json!({}));
        if !attributes.is_object() {
            *attributes = json!({});
        }
        let Some(attributes) = attributes.as_object_mut() else {
            return Err(KeycloakError::Unexpected(
                "invalid Keycloak user attributes".to_owned(),
            ));
        };
        attributes.insert(name.to_owned(), json!([value]));
        let response = self
            .inner
            .http
            .put(&url)
            .bearer_auth(token)
            .json(&body)
            .send()
            .await
            .map_err(transport)?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(status_error(response.status()))
        }
    }

    /// Create a development user only when the exact username is absent.
    /// Existing accounts are returned without any update, including password
    /// changes. A concurrent create is resolved by looking up the winner.
    pub async fn create_user_if_missing(
        &self,
        username: &str,
        email: &str,
        display_name: &str,
        password: &str,
    ) -> Result<(String, bool), KeycloakError> {
        if let Some(id) = self.find_user_id(username).await? {
            return Ok((id, false));
        }

        let token = self.access_token().await?;
        let (first_name, last_name) = display_name
            .split_once(' ')
            .map_or((display_name, ""), |(first, rest)| (first, rest));
        let url = format!(
            "{}/admin/realms/{}/users",
            self.inner.settings.base_url.trim_end_matches('/'),
            self.inner.settings.realm
        );
        let response = self
            .inner
            .http
            .post(url)
            .bearer_auth(token)
            .json(&json!({
                "username": username,
                "email": email,
                "emailVerified": true,
                "firstName": first_name,
                "lastName": last_name,
                "enabled": true,
                "credentials": [{"type": "password", "value": password, "temporary": false}]
            }))
            .send()
            .await
            .map_err(transport)?;

        if response.status().is_success() {
            if let Some(id) = response
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|location| location.to_str().ok())
                .and_then(|location| location.rsplit('/').next())
            {
                return Ok((id.to_owned(), true));
            }
            let id = self.find_user_id(username).await?.ok_or_else(|| {
                KeycloakError::Unexpected("created user could not be read back".to_owned())
            })?;
            return Ok((id, true));
        }

        if response.status() == reqwest::StatusCode::CONFLICT {
            let id = self
                .find_user_id(username)
                .await?
                .ok_or_else(|| status_error(reqwest::StatusCode::CONFLICT))?;
            return Ok((id, false));
        }
        Err(status_error(response.status()))
    }

    async fn find_user_id(&self, username: &str) -> Result<Option<String>, KeycloakError> {
        let token = self.access_token().await?;
        let url = format!(
            "{}/admin/realms/{}/users",
            self.inner.settings.base_url.trim_end_matches('/'),
            self.inner.settings.realm
        );
        let response = self
            .inner
            .http
            .get(url)
            .bearer_auth(token)
            .query(&[("username", username), ("exact", "true")])
            .send()
            .await
            .map_err(transport)?;
        if !response.status().is_success() {
            return Err(status_error(response.status()));
        }
        let users: Vec<AdminUser> = response.json().await.map_err(|error| {
            KeycloakError::Unexpected(format!("decode admin users response: {error}"))
        })?;
        Ok(users
            .into_iter()
            .find(|user| user.username.as_deref() == Some(username))
            .and_then(|user| user.id))
    }

    /// Return a cached admin token, fetching a fresh one when it is near expiry.
    async fn access_token(&self) -> Result<String, KeycloakError> {
        {
            let guard = self.inner.token.lock().await;
            if let Some(token) = guard.as_ref()
                && token.expires_at > Instant::now() + TOKEN_REFRESH_SKEW
            {
                return Ok(token.value.clone());
            }
        }

        // Hold the lock across the fetch so concurrent callers coalesce.
        let mut guard = self.inner.token.lock().await;
        if let Some(token) = guard.as_ref()
            && token.expires_at > Instant::now() + TOKEN_REFRESH_SKEW
        {
            return Ok(token.value.clone());
        }

        let url = format!(
            "{}/realms/{}/protocol/openid-connect/token",
            self.inner.settings.base_url.trim_end_matches('/'),
            self.inner.settings.realm
        );
        let response = self
            .inner
            .http
            .post(url)
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", self.inner.settings.client_id.as_str()),
                ("client_secret", self.inner.settings.client_secret.as_str()),
            ])
            .send()
            .await
            .map_err(transport)?;
        if !response.status().is_success() {
            return Err(status_error(response.status()));
        }
        let token: TokenResponse = response.json().await.map_err(|error| {
            KeycloakError::Unexpected(format!("decode token response: {error}"))
        })?;
        let lifetime = Duration::from_secs(token.expires_in.unwrap_or(60));
        *guard = Some(CachedToken {
            value: token.access_token.clone(),
            expires_at: Instant::now() + lifetime,
        });
        Ok(token.access_token)
    }
}

#[derive(Deserialize)]
struct AdminUser {
    id: Option<String>,
    username: Option<String>,
}

fn transport(error: reqwest::Error) -> KeycloakError {
    KeycloakError::Unavailable(error.to_string())
}

fn status_error(status: reqwest::StatusCode) -> KeycloakError {
    match status {
        reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => {
            KeycloakError::Unauthorized(format!("admin request rejected with {status}"))
        }
        _ => KeycloakError::Unexpected(format!("admin request failed with {status}")),
    }
}
