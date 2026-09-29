use prometheus::{IntCounterVec, IntGauge, Opts, Registry};
#[derive(Clone)]
pub(super) struct RecordingMetrics {
    pub active: IntGauge,
    pub spool: IntGauge,
    pub failures: IntCounterVec,
    pub uploads: IntCounterVec,
}
impl RecordingMetrics {
    pub fn register(registry: &Registry) -> Result<Self, prometheus::Error> {
        let metrics = Self {
            active: IntGauge::new(
                "mahfel_recording_active_generations",
                "Active local recording capture generations",
            )?,
            spool: IntGauge::new(
                "mahfel_recording_spool_bytes",
                "Local recording spool bytes awaiting durable upload",
            )?,
            failures: IntCounterVec::new(
                Opts::new(
                    "mahfel_recording_failures_total",
                    "Recording capture interruptions",
                ),
                &["reason"],
            )?,
            uploads: IntCounterVec::new(
                Opts::new(
                    "mahfel_recording_upload_cycles_total",
                    "Recording upload cycle outcomes",
                ),
                &["outcome"],
            )?,
        };
        registry.register(Box::new(metrics.active.clone()))?;
        registry.register(Box::new(metrics.spool.clone()))?;
        registry.register(Box::new(metrics.failures.clone()))?;
        registry.register(Box::new(metrics.uploads.clone()))?;
        Ok(metrics)
    }
}
