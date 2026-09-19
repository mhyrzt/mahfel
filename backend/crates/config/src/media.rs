use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct MediaSettings {
    pub grpc_bind: String,
    pub operations_bind: String,
    pub worker_count: usize,
    pub rtc_port_min: u16,
    pub rtc_port_max: u16,
    pub listen_ip: String,
    pub announced_address: String,
}
