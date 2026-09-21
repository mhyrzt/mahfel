//! Keycloak access-token verification (ADR 0019).
//!
//! `mahfel-api` is a resource server: it verifies issuer, audience, expiry, and
//! signature (JWKS with key rotation) before trusting any claim. The verified
//! identity is provider-neutral; provider-specific claim containers are mapped
//! here at the boundary.

use serde::Deserialize;
use serde_json::Value;
use std::{collections::BTreeSet, time::Duration};
use thiserror::Error;

/// Provider-neutral authenticated identity derived from a verified token.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthenticatedIdentity {
    pub subject: String,
    /// Verified issuer this token was issued by, as configured.
    pub issuer: String,
    /// Local application user id, linked by provisioning and injected as a
    /// claim. Absent until the user has provisioned.
    pub app_user_id: Option<String>,
    /// Optional principal kind claim (`user` or `service`).
    ///
    /// Absent means `user`. Unknown values are normalized to `None` (treated as
    /// `user`) so a malformed claim can never widen access to a machine actor.
    pub app_principal_kind: Option<String>,
    pub email: Option<String>,
    /// Whether the provider asserts the email is verified.
    pub email_verified: bool,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub roles: Vec<String>,
}

/// The acting principal's kind, as normalized from the provider claims.
///
/// Education- and transport-agnostic: authorization consumes this instead of
/// re-parsing the raw claim string.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PrincipalKind {
    #[default]
    User,
    Service,
}

impl AuthenticatedIdentity {
    /// The principal kind this token asserts.
    ///
    /// An absent claim, or a value that is not `user` or `service`, resolves to
    /// [`PrincipalKind::User`] so a malformed claim can never widen access to a
    /// machine actor.
    #[must_use]
    pub fn kind(&self) -> PrincipalKind {
        match self.app_principal_kind.as_deref() {
            Some("service") => PrincipalKind::Service,
            _ => PrincipalKind::User,
        }
    }
}

#[derive(Debug, Error)]
pub enum ClaimsError {
    #[error("OIDC claims do not contain a subject")]
    MissingSubject,
    #[error("OIDC claims could not be decoded: {0}")]
    Invalid(#[from] serde_json::Error),
}

/// Discovery document subset needed for verification.
#[derive(Clone, Debug, Deserialize)]
pub struct DiscoveryDocument {
    pub issuer: String,
    pub jwks_uri: String,
}

/// Verification failure.
#[derive(Debug, Error)]
pub enum TokenError {
    #[error("token is malformed")]
    Malformed,
    #[error("token signature is invalid")]
    InvalidSignature,
    #[error("token has expired")]
    Expired,
    #[error("token is not valid yet")]
    NotYetValid,
    #[error("token was issued for another issuer")]
    WrongIssuer,
    #[error("token was issued for another audience")]
    WrongAudience,
    #[error("token has no usable signing key")]
    UnknownKey,
    #[error("verification key set is unavailable")]
    KeysUnavailable,
    #[error("provider metadata is unavailable")]
    ProviderUnavailable,
    #[error("token could not be decoded: {0}")]
    Decode(String),
}

/// Clock skew tolerated when validating `exp`/`nbf`.
pub const DEFAULT_LEEWAY: Duration = Duration::from_secs(30);

#[derive(Debug, Deserialize)]
struct StandardClaims {
    sub: Option<String>,
    app_user_id: Option<String>,
    app_principal_kind: Option<String>,
    email: Option<String>,
    email_verified: Option<bool>,
    preferred_username: Option<String>,
    name: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
    picture: Option<String>,
    groups: Option<Vec<String>>,
    realm_access: Option<RoleContainer>,
    resource_access: Option<std::collections::HashMap<String, RoleContainer>>,
}

#[derive(Debug, Deserialize)]
struct RoleContainer {
    roles: Option<Vec<String>>,
}

/// Converts standard OIDC claims and bounded provider role containers into
/// Mahfel's identity model.
///
/// Authorization code consumes this type instead of inspecting provider JSON.
pub fn normalize_claims(claims: &Value) -> Result<AuthenticatedIdentity, ClaimsError> {
    let claims: StandardClaims = serde_json::from_value(claims.clone())?;
    let subject = claims
        .sub
        .filter(|value| !value.is_empty())
        .ok_or(ClaimsError::MissingSubject)?;

    let mut roles = BTreeSet::new();
    if let Some(groups) = claims.groups {
        roles.extend(groups);
    }
    if let Some(container) = claims.realm_access
        && let Some(values) = container.roles
    {
        roles.extend(values);
    }
    if let Some(resources) = claims.resource_access {
        for container in resources.into_values() {
            if let Some(values) = container.roles {
                roles.extend(values);
            }
        }
    }

    let display_name = claims.name.or_else(|| {
        let joined = [claims.given_name, claims.family_name]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ");
        (!joined.is_empty()).then_some(joined)
    });

    let email_verified = claims.email_verified.unwrap_or(false) && claims.email.is_some();

    Ok(AuthenticatedIdentity {
        subject,
        issuer: String::new(),
        app_user_id: claims.app_user_id.filter(|value| !value.is_empty()),
        app_principal_kind: claims
            .app_principal_kind
            .filter(|value| matches!(value.as_str(), "user" | "service")),
        email: claims.email,
        // A verified flag without an email is meaningless.
        email_verified,
        username: claims.preferred_username,
        display_name,
        avatar_url: claims.picture,
        roles: roles.into_iter().collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::PrincipalKind;
    use super::normalize_claims;
    use serde_json::json;

    #[test]
    fn normalizes_standard_claims() {
        let identity = normalize_claims(&json!({
            "sub": "student",
            "email": "student@mahfel.local",
            "preferred_username": "student",
            "name": "Student"
        }))
        .expect("valid claims");

        assert_eq!(identity.subject, "student");
        assert_eq!(identity.roles, Vec::<String>::new());
        assert_eq!(identity.display_name.as_deref(), Some("Student"));
    }

    #[test]
    fn normalizes_keycloak_role_containers_at_the_boundary() {
        let identity = normalize_claims(&json!({
            "sub": "teacher",
            "realm_access": {"roles": ["teacher"]},
            "resource_access": {"mahfel": {"roles": ["classroom:manage"]}}
        }))
        .expect("valid claims");

        assert_eq!(identity.roles, vec!["classroom:manage", "teacher"]);
    }

    #[test]
    fn derives_a_display_name_from_given_and_family_names() {
        let identity = normalize_claims(&json!({
            "sub": "teacher",
            "given_name": "Misato",
            "family_name": "Katsuragi"
        }))
        .expect("valid claims");

        assert_eq!(identity.display_name.as_deref(), Some("Misato Katsuragi"));
    }

    #[test]
    fn normalizes_the_linked_app_user_id() {
        let identity = normalize_claims(&json!({
            "sub": "teacher",
            "app_user_id": "11111111-1111-4111-8111-111111111111"
        }))
        .expect("valid claims");
        assert_eq!(
            identity.app_user_id.as_deref(),
            Some("11111111-1111-4111-8111-111111111111")
        );

        let unlinked = normalize_claims(&json!({ "sub": "teacher", "app_user_id": "" }))
            .expect("valid claims");
        assert!(unlinked.app_user_id.is_none());
    }

    #[test]
    fn normalizes_the_optional_principal_kind() {
        let absent = normalize_claims(&json!({ "sub": "teacher" })).expect("valid claims");
        assert!(absent.app_principal_kind.is_none());

        let service = normalize_claims(&json!({
            "sub": "lms-core",
            "app_principal_kind": "service"
        }))
        .expect("valid claims");
        assert_eq!(service.app_principal_kind.as_deref(), Some("service"));

        let user = normalize_claims(&json!({
            "sub": "teacher",
            "app_principal_kind": "user"
        }))
        .expect("valid claims");
        assert_eq!(user.app_principal_kind.as_deref(), Some("user"));

        // An unknown kind is dropped (treated as user), never trusted.
        let unknown = normalize_claims(&json!({
            "sub": "teacher",
            "app_principal_kind": "admin"
        }))
        .expect("valid claims");
        assert!(unknown.app_principal_kind.is_none());
        assert_eq!(unknown.kind(), PrincipalKind::User);
    }

    #[test]
    fn kind_defaults_to_user_and_only_accepts_service() {
        let absent = normalize_claims(&json!({ "sub": "teacher" })).expect("valid claims");
        assert_eq!(absent.kind(), PrincipalKind::User);

        let explicit_user =
            normalize_claims(&json!({ "sub": "teacher", "app_principal_kind": "user" }))
                .expect("valid claims");
        assert_eq!(explicit_user.kind(), PrincipalKind::User);

        let service = normalize_claims(&json!({
            "sub": "lms-core",
            "app_principal_kind": "service"
        }))
        .expect("valid claims");
        assert_eq!(service.kind(), PrincipalKind::Service);

        // A malformed claim must never resolve to a machine actor.
        for hostile in ["admin", "SERVICE", "Service", "", "service "] {
            let identity =
                normalize_claims(&json!({ "sub": "teacher", "app_principal_kind": hostile }))
                    .expect("valid claims");
            assert_eq!(
                identity.kind(),
                PrincipalKind::User,
                "malformed kind {hostile:?} must not widen access"
            );
        }
    }
}
