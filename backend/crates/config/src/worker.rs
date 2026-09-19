use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct WorkerSettings {
    pub operations_bind: String,
    pub amqp_url: String,
}
