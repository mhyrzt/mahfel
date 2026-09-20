use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct TurnSettings {
    pub static_secret: String,
    pub realm: String,
    pub uris: Vec<String>,
    pub ttl_seconds: u64,
}
