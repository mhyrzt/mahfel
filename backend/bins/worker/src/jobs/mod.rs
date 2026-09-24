mod consumer;
mod handler;
mod metrics;
mod processor;
mod relay;
mod runtime;

#[cfg(test)]
mod tests;

pub use metrics::JobMetrics;
pub use runtime::run;

use std::time::Duration;

const RECONNECT_DELAY: Duration = Duration::from_secs(5);
const OUTBOX_POLL: Duration = Duration::from_secs(1);
const CLAIM_RENEW: Duration = Duration::from_secs(30);
