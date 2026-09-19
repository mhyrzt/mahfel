use prometheus::{IntCounter, IntCounterVec, Opts, Registry};

/// Redis adapter counters. Labels stay bounded: operations and outcomes come
/// from a closed set.
#[derive(Clone)]
pub struct RedisMetrics {
    commands_total: IntCounterVec,
    pubsub_messages_total: IntCounterVec,
    probe_failures_total: IntCounter,
}

impl RedisMetrics {
    pub fn register(registry: &Registry) -> Result<Self, prometheus::Error> {
        let commands_total = IntCounterVec::new(
            Opts::new("redis_commands_total", "Redis commands by outcome"),
            &["outcome"],
        )?;
        let pubsub_messages_total = IntCounterVec::new(
            Opts::new(
                "redis_pubsub_messages_total",
                "Redis pub/sub messages by direction",
            ),
            &["direction"],
        )?;
        let probe_failures_total = IntCounter::with_opts(Opts::new(
            "redis_probe_failures_total",
            "Total failed Redis readiness probes",
        ))?;

        registry.register(Box::new(commands_total.clone()))?;
        registry.register(Box::new(pubsub_messages_total.clone()))?;
        registry.register(Box::new(probe_failures_total.clone()))?;

        Ok(Self {
            commands_total,
            pubsub_messages_total,
            probe_failures_total,
        })
    }

    pub fn command_ok(&self) {
        self.commands_total.with_label_values(&["ok"]).inc();
    }

    pub fn command_error(&self) {
        self.commands_total.with_label_values(&["error"]).inc();
    }

    pub fn message_published(&self) {
        self.pubsub_messages_total
            .with_label_values(&["published"])
            .inc();
    }

    pub fn message_received(&self) {
        self.pubsub_messages_total
            .with_label_values(&["received"])
            .inc();
    }

    pub fn probe_failure(&self) {
        self.probe_failures_total.inc();
    }
}
