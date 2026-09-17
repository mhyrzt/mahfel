use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeSet;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthenticatedIdentity {
    pub subject: String,
    pub email: Option<String>,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Debug, Error)]
pub enum ClaimsError {
    #[error("OIDC claims do not contain a subject")]
    MissingSubject,
    #[error("OIDC claims could not be decoded: {0}")]
    Invalid(#[from] serde_json::Error),
}

#[derive(Debug, Deserialize)]
struct StandardClaims {
    sub: Option<String>,
    email: Option<String>,
    preferred_username: Option<String>,
    name: Option<String>,
    groups: Option<Vec<String>>,
    realm_access: Option<RoleContainer>,
    resource_access: Option<std::collections::HashMap<String, RoleContainer>>,
}

#[derive(Debug, Deserialize)]
struct RoleContainer {
    roles: Option<Vec<String>>,
}

/// Converts standard OIDC claims and bounded provider role containers into Mahfel's identity model.
/// Authorization code should consume this type instead of inspecting provider-specific JSON.
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

    Ok(AuthenticatedIdentity {
        subject,
        email: claims.email,
        username: claims.preferred_username,
        display_name: claims.name,
        roles: roles.into_iter().collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::normalize_claims;
    use serde_json::json;

    #[test]
    fn normalizes_dex_standard_claims() {
        let identity = normalize_claims(&json!({
            "sub": "student",
            "email": "student@mahfel.local",
            "preferred_username": "student",
            "name": "Student"
        }))
        .expect("valid claims");

        assert_eq!(identity.subject, "student");
        assert_eq!(identity.roles, Vec::<String>::new());
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
}
