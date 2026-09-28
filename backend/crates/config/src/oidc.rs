use std::net::IpAddr;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct OidcSettings {
    pub issuer_url: String,
    #[serde(default)]
    pub issuer_resolve_ip: Option<IpAddr>,
    #[serde(default)]
    pub issuer_ca_cert_path: Option<String>,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}
