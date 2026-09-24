use anyhow::Result;
use prometheus::{Gauge, IntCounterVec, IntGauge, Opts, Registry};

#[derive(Clone)]
pub struct JobMetrics {
    pub(super) publishes: IntCounterVec,
    pub(super) outcomes: IntCounterVec,
    pub(super) pending: IntGauge,
    pub(super) oldest: Gauge,
    pub(super) active: IntGauge,
}

impl JobMetrics {
    pub fn register(registry: &Registry) -> Result<Self, prometheus::Error> {
        let publishes = IntCounterVec::new(
            Opts::new("job_publishes_total", "Job publish outcomes"),
            &["outcome"],
        )?;
        let outcomes = IntCounterVec::new(
            Opts::new("job_outcomes_total", "Worker job outcomes"),
            &["outcome"],
        )?;
        let pending = IntGauge::with_opts(Opts::new(
            "job_outbox_pending",
            "Due unpublished outbox rows",
        ))?;
        let oldest = Gauge::with_opts(Opts::new(
            "job_outbox_oldest_seconds",
            "Age of the oldest due outbox row",
        ))?;
        let active = IntGauge::with_opts(Opts::new("jobs_active", "Active worker jobs"))?;
        registry.register(Box::new(publishes.clone()))?;
        registry.register(Box::new(outcomes.clone()))?;
        registry.register(Box::new(pending.clone()))?;
        registry.register(Box::new(oldest.clone()))?;
        registry.register(Box::new(active.clone()))?;
        Ok(Self {
            publishes,
            outcomes,
            pending,
            oldest,
            active,
        })
    }
}

pub(super) struct ActiveJob(pub(super) IntGauge);
impl Drop for ActiveJob {
    fn drop(&mut self) {
        self.0.dec();
    }
}
