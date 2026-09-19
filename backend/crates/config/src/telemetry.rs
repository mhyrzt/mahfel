use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct TelemetrySettings {
    pub log_filter: String,
    pub otlp_endpoint: String,
}
