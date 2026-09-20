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
        let body = json!({ "attributes": { name: [value] } });
        let response = self
            .inner
            .http
            .put(url)
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
