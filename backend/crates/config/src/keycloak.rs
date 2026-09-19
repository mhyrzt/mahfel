use serde::Deserialize;

/// Keycloak Admin API access for provisioning and user management.
///
/// `mahfel-api` authenticates with the client-credentials grant of a dedicated
/// service-account client; it never uses an interactive administrator account.
#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct KeycloakSettings {
    /// Public base URL of the Keycloak deployment, without a realm path.
    pub base_url: String,
    pub realm: String,
    pub admin_client_id: String,
    pub admin_client_secret: String,
}

impl Default for KeycloakSettings {
    fn default() -> Self {
        Self {
            base_url: "https://auth.mahfel.localhost".to_owned(),
            realm: "rastai".to_owned(),
            admin_client_id: "mahfel-admin".to_owned(),
            admin_client_secret: "mahfel-admin-dev-secret".to_owned(),
        }
    }
}
