//! JSON Web Key Set fetching with key rotation.
//!
//! The key set is cached in memory and refreshed when a token references an
//! unknown `kid`, so key rotation does not require a restart. Concurrent
//! refreshes are coalesced. RSA public keys are validated and handed to
//! `jsonwebtoken` via `DecodingKey::from_rsa_components`, so this module does
//! not reimplement DER or base64url.

use crate::TokenError;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use jsonwebtoken::jwk::{AlgorithmParameters, JwkSet, PublicKeyUse};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

/// How long a fetched key set is considered fresh.
pub const DEFAULT_KEY_TTL: Duration = Duration::from_secs(300);

/// How long to wait for provider metadata or a key set.
pub const DEFAULT_HTTP_TIMEOUT: Duration = Duration::from_secs(5);

/// A verification key plus the algorithm its JWK declares.
#[derive(Clone, Debug)]
pub struct VerificationKey {
    pub n: String,
    pub e: String,
    pub algorithm: Option<String>,
}

/// A key set indexed by key id.
#[derive(Clone, Debug, Default)]
pub struct KeySet {
    keys: HashMap<String, VerificationKey>,
}

impl KeySet {
    /// Find an RSA public key by `kid`.
    #[must_use]
    pub fn rsa_key(&self, kid: &str) -> Option<&VerificationKey> {
        self.keys.get(kid)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}

struct Cached {
    keys: KeySet,
    fetched_at: Instant,
}

/// Fetches and caches the provider's RSA verification keys.
pub struct JwksCache {
    jwks_uri: String,
    http: reqwest::Client,
    ttl: Duration,
    cached: RwLock<Option<Cached>>,
    refresh: Mutex<()>,
}

impl JwksCache {
    #[must_use]
    pub fn new(jwks_uri: impl Into<String>, http: reqwest::Client, ttl: Duration) -> Self {
        Self {
            jwks_uri: jwks_uri.into(),
            http,
            ttl,
            cached: RwLock::new(None),
            refresh: Mutex::new(()),
        }
    }

    /// Return a cached key, refreshing when stale or when `kid` is unknown.
    pub async fn key(&self, kid: &str) -> Result<VerificationKey, TokenError> {
        if let Some(key) = self.cached_key(kid) {
            return Ok(key);
        }
        let _guard = self.refresh.lock().await;
        if let Some(key) = self.cached_key(kid) {
            return Ok(key);
        }
        self.fetch().await?;
        self.cached_key(kid).ok_or(TokenError::UnknownKey)
    }

    fn cached_key(&self, kid: &str) -> Option<VerificationKey> {
        let guard = self.cached.read().ok()?;
        let cached = guard.as_ref()?;
        if cached.fetched_at.elapsed() >= self.ttl {
            return None;
        }
        cached.keys.rsa_key(kid).cloned()
    }

    async fn fetch(&self) -> Result<(), TokenError> {
        let document: JwkSet = self
            .http
            .get(&self.jwks_uri)
            .send()
            .await
            .map_err(|_| TokenError::KeysUnavailable)?
            .error_for_status()
            .map_err(|_| TokenError::KeysUnavailable)?
            .json()
            .await
            .map_err(|_| TokenError::KeysUnavailable)?;

        let mut keys = HashMap::new();
        for jwk in document.keys {
            let AlgorithmParameters::RSA(params) = jwk.algorithm else {
                continue;
            };
            let Some(kid) = jwk.common.key_id else {
                continue;
            };
            if let Some(use_) = jwk.common.public_key_use
                && use_ != PublicKeyUse::Signature
            {
                continue;
            }
            let alg = jwk
                .common
                .key_algorithm
                .map(|alg| format!("{alg:?}"))
                .or(None);
            if let Some(alg) = &alg
                && alg != "RS256"
                && !alg.contains("RS256")
            {
                continue;
            }
            if !is_strong_rsa_key(&params.n) {
                continue;
            }
            if jsonwebtoken::DecodingKey::from_rsa_components(&params.n, &params.e).is_err() {
                continue;
            }
            keys.insert(
                kid,
                VerificationKey {
                    n: params.n,
                    e: params.e,
                    algorithm: jwk.common.key_algorithm.map(|a| format!("{a:?}")).or(alg),
                },
            );
        }

        let mut guard = self
            .cached
            .write()
            .map_err(|_| TokenError::KeysUnavailable)?;
        *guard = Some(Cached {
            keys: KeySet { keys },
            fetched_at: Instant::now(),
        });
        Ok(())
    }
}

/// Memory cache shared by verification calls.
pub type SharedJwks = Arc<JwksCache>;

/// Minimum RSA modulus size accepted from a provider key.
const MIN_RSA_BITS: usize = 2048;

fn is_strong_rsa_key(modulus_b64url: &str) -> bool {
    let Ok(bytes) = URL_SAFE_NO_PAD.decode(modulus_b64url) else {
        return false;
    };
    let mut bytes = bytes;
    while bytes.first() == Some(&0) {
        bytes.remove(0);
    }
    bytes.len() * 8 >= MIN_RSA_BITS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_small_moduli() {
        assert!(
            !is_strong_rsa_key(&encode_base64url(&[0x01u8; 64])),
            "a modulus below 2048 bits must be rejected"
        );
        assert!(is_strong_rsa_key(&encode_base64url(&[0x01u8; 256])));
    }

    /// Key rotation: a token signed by a newly published key must trigger a
    /// key-set refresh rather than failing until the cache TTL expires.
    #[tokio::test]
    async fn refetches_when_the_key_id_rotates() {
        use std::sync::{Arc, RwLock};

        let active = Arc::new(RwLock::new("first".to_owned()));
        let state = Arc::clone(&active);
        let app = axum::Router::new().route(
            "/jwks",
            axum::routing::get(move || {
                let state = Arc::clone(&state);
                async move {
                    let kid = state.read().expect("lock").clone();
                    axum::Json(serde_json::json!({
                        "keys": [{
                            "kty": "RSA",
                            "kid": kid,
                            "alg": "RS256",
                            "use": "sig",
                            "n": build_modulus(),
                            "e": build_exponent(),
                        }]
                    }))
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let address = listener.local_addr().expect("address");
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });

        let cache = JwksCache::new(
            format!("http://{address}/jwks"),
            reqwest::Client::new(),
            DEFAULT_KEY_TTL,
        );
        assert!(cache.key("first").await.is_ok());

        *active.write().expect("lock") = "second".to_owned();
        assert!(
            cache.key("second").await.is_ok(),
            "an unknown key id must trigger a refresh"
        );
        assert!(
            matches!(cache.key("first").await, Err(TokenError::UnknownKey)),
            "the refreshed key set replaces the rotated-out key"
        );
    }

    fn build_modulus() -> String {
        encode_base64url(&[0x01u8; 256])
    }

    fn build_exponent() -> String {
        encode_base64url(&[0x01, 0x00, 0x01])
    }

    fn encode_base64url(value: &[u8]) -> String {
        URL_SAFE_NO_PAD.encode(value)
    }
}
