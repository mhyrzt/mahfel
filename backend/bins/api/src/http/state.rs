use super::auth::AuthState;

/// Shared router state for the product API.
#[derive(Clone)]
pub struct AppState {
    pub service: &'static str,
    pub version: &'static str,
    pub node_id: String,
    pub auth: AuthState,
}
