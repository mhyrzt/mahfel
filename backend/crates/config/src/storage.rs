use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct StorageSettings {
    pub internal_endpoint: String,
    pub public_endpoint: String,
    pub region: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    #[serde(default = "default_path_style")]
    pub force_path_style: bool,
}

const fn default_path_style() -> bool {
    true
}
