use std::time::Duration;

/// Connection settings for the Keycloak Admin API.
///
/// The adapter authenticates with the client-credentials grant of a dedicated
/// service account; it never uses an interactive administrator account.
#[derive(Clone, Debug)]
pub struct KeycloakAdminSettings {
    /// Public base URL of the deployment, without a realm path.
    pub base_url: String,
    pub realm: String,
    pub client_id: String,
    pub client_secret: String,
    pub http_timeout: Duration,
}
