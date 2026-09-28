//! Access-token verification (ADR 0019).
//!
//! A verifier is bound to one issuer and accepted audience. It fetches discovery
//! metadata lazily, validates tokens against the provider's RSA keys, and
//! returns a provider-neutral [`AuthenticatedIdentity`].

use crate::{
    AuthenticatedIdentity, ClaimsError, DiscoveryDocument, TokenError,
    jwks::{DEFAULT_HTTP_TIMEOUT, DEFAULT_KEY_TTL, JwksCache, SharedJwks},
    normalize_claims,
};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// Verification configuration for one provider.
#[derive(Clone, Debug)]
pub struct TokenVerifierConfig {
    pub issuer_url: String,
    /// Optional address used to resolve the issuer host for discovery and JWKS requests.
    pub issuer_resolve_ip: Option<IpAddr>,
    /// Additional trusted root for issuer discovery and JWKS requests.
    pub issuer_ca_cert: Option<reqwest::Certificate>,
    /// Audience or authorized party the token must be issued for.
    pub audience: String,
    /// Clock skew tolerated when validating time claims.
    pub leeway: Duration,
    /// How long fetched signing keys stay fresh.
    pub key_ttl: Duration,
    /// Timeout for provider metadata and key requests.
    pub http_timeout: Duration,
}

impl TokenVerifierConfig {
    #[must_use]
    pub fn new(issuer_url: impl Into<String>, audience: impl Into<String>) -> Self {
        Self {
            issuer_url: issuer_url.into(),
            issuer_resolve_ip: None,
            issuer_ca_cert: None,
            audience: audience.into(),
            leeway: crate::DEFAULT_LEEWAY,
            key_ttl: DEFAULT_KEY_TTL,
            http_timeout: DEFAULT_HTTP_TIMEOUT,
        }
    }

    /// Add a PEM-encoded local root used to validate the configured issuer.
    pub fn with_issuer_ca_cert(mut self, pem: &[u8]) -> Result<Self, reqwest::Error> {
        self.issuer_ca_cert = Some(reqwest::Certificate::from_pem(pem)?);
        Ok(self)
    }
}

/// Verifies provider access tokens for one issuer/audience pair.
pub struct TokenVerifier {
    audience: String,
    leeway: Duration,
    http: reqwest::Client,
    issuer_url: String,
    discovery: tokio::sync::Mutex<Option<DiscoveryDocument>>,
    keys: tokio::sync::Mutex<Option<SharedJwks>>,
}

impl TokenVerifier {
    #[must_use]
    pub fn new(config: TokenVerifierConfig) -> Self {
        let issuer_url = config.issuer_url.trim_end_matches('/').to_owned();
        let mut http = reqwest::Client::builder().timeout(config.http_timeout);
        if let Some(certificate) = config.issuer_ca_cert {
            http = http.add_root_certificate(certificate);
        }
        if let Some(resolve_ip) = config.issuer_resolve_ip
            && let Ok(issuer) = reqwest::Url::parse(&issuer_url)
            && let (Some(host), Some(port)) = (issuer.host_str(), issuer.port_or_known_default())
        {
            http = http.resolve(host, SocketAddr::new(resolve_ip, port));
        }
        Self {
            audience: config.audience,
            leeway: config.leeway,
            http: http.build().unwrap_or_default(),
            issuer_url,
            discovery: tokio::sync::Mutex::new(None),
            keys: tokio::sync::Mutex::new(None),
        }
    }

    /// Verify a bearer token and normalize its claims.
    pub async fn verify(&self, token: &str) -> Result<AuthenticatedIdentity, TokenError> {
        let header = decode_header(token)?;
        if header.alg != Algorithm::RS256 {
            return Err(TokenError::InvalidSignature);
        }

        let document = self.discovery().await?;
        if document.issuer.trim_end_matches('/') != self.issuer_url {
            return Err(TokenError::WrongIssuer);
        }
        let keys = self.jwks(&document).await;
        let key = match header.kid {
            Some(kid) => keys.key(&kid).await?,
            None => return Err(TokenError::UnknownKey),
        };
        // A JWK may pin its algorithm; reject a mismatch rather than trusting
        // the token header alone.
        if key
            .algorithm
            .as_deref()
            .is_some_and(|algorithm| !algorithm.to_uppercase().contains("RS256"))
        {
            return Err(TokenError::InvalidSignature);
        }

        let decoding_key = DecodingKey::from_rsa_components(&key.n, &key.e)
            .map_err(|_| TokenError::InvalidSignature)?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[self.issuer_url.as_str()]);
        validation.set_audience(&[self.audience.as_str()]);
        validation.leeway = self.leeway.as_secs();
        validation.validate_exp = true;
        // Keycloak access tokens use `aud` for the resource and `azp` for the
        // client; both are accepted so a service token cannot be replayed.
        validation.validate_nbf = true;

        let decoded = jsonwebtoken::decode::<serde_json::Value>(token, &decoding_key, &validation)
            .map_err(classify)?;

        let mut identity = normalize_claims(&decoded.claims).map_err(claims_error)?;
        // The verified issuer, not a claim, is authoritative for the identity key.
        identity.issuer = self.issuer_url.clone();
        Ok(identity)
    }

    async fn discovery(&self) -> Result<DiscoveryDocument, TokenError> {
        let mut guard = self.discovery.lock().await;
        if let Some(document) = guard.as_ref() {
            return Ok(document.clone());
        }
        let document: DiscoveryDocument = self
            .http
            .get(format!(
                "{}/.well-known/openid-configuration",
                self.issuer_url
            ))
            .send()
            .await
            .map_err(|_| TokenError::ProviderUnavailable)?
            .error_for_status()
            .map_err(|_| TokenError::ProviderUnavailable)?
            .json()
            .await
            .map_err(|_| TokenError::ProviderUnavailable)?;
        *guard = Some(document.clone());
        Ok(document)
    }

    async fn jwks(&self, document: &DiscoveryDocument) -> SharedJwks {
        let mut guard = self.keys.lock().await;
        if let Some(keys) = guard.as_ref() {
            return Arc::clone(keys);
        }
        let keys = Arc::new(JwksCache::new(
            document.jwks_uri.clone(),
            self.http.clone(),
            DEFAULT_KEY_TTL,
        ));
        *guard = Some(Arc::clone(&keys));
        keys
    }
}

#[derive(Debug, Deserialize)]
struct TokenHeader {
    #[serde(deserialize_with = "deserialize_algorithm")]
    alg: Algorithm,
    kid: Option<String>,
}

fn deserialize_algorithm<'de, D>(deserializer: D) -> Result<Algorithm, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    value.parse().map_err(serde::de::Error::custom)
}

fn decode_header(token: &str) -> Result<TokenHeader, TokenError> {
    let encoded = token.split('.').next().ok_or(TokenError::Malformed)?;
    let bytes = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| TokenError::Malformed)?;
    serde_json::from_slice(&bytes).map_err(|_| TokenError::Malformed)
}

fn classify(error: jsonwebtoken::errors::Error) -> TokenError {
    use jsonwebtoken::errors::ErrorKind;
    match error.kind() {
        ErrorKind::ExpiredSignature => TokenError::Expired,
        ErrorKind::ImmatureSignature => TokenError::NotYetValid,
        ErrorKind::InvalidIssuer => TokenError::WrongIssuer,
        ErrorKind::InvalidAudience => TokenError::WrongAudience,
        ErrorKind::InvalidSignature => TokenError::InvalidSignature,
        other => TokenError::Decode(format!("{other:?}")),
    }
}

fn claims_error(error: ClaimsError) -> TokenError {
    TokenError::Decode(error.to_string())
}

/// Current UNIX time in seconds. Exposed for tests that mint tokens.
#[must_use]
pub fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}
