use mahfel_rtc::MediaRuntime;
use mahfel_telemetry::HealthState;
use prometheus::{Counter, IntGauge, Opts, Registry};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;

use crate::validation::saturating_i64;

#[derive(Clone)]
pub(crate) struct MediaMetrics {
    pub(crate) active_workers: IntGauge,
    pub(crate) worker_failures: Counter,
    pub(crate) active_rooms: IntGauge,
    pub(crate) active_transports: IntGauge,
    pub(crate) active_producers: IntGauge,
    pub(crate) active_consumers: IntGauge,
    pub(crate) active_speakers: IntGauge,
    pub(crate) speaker_stream_dropped_updates: Counter,
}

impl MediaMetrics {
    pub(crate) fn register(health: &HealthState) -> Result<Self, prometheus::Error> {
        let registry: Registry = health.registry();
        let active_workers = IntGauge::with_opts(Opts::new(
            "media_active_workers",
            "Number of mediasoup workers currently not closed",
        ))?;
        registry.register(Box::new(active_workers.clone()))?;

        let worker_failures = Counter::with_opts(Opts::new(
            "media_worker_failures_total",
            "Total observed mediasoup worker failures (closed workers)",
        ))?;
        registry.register(Box::new(worker_failures.clone()))?;

        let active_rooms = IntGauge::with_opts(Opts::new(
            "media_active_rooms",
            "Number of active mediasoup routers (rooms)",
        ))?;
        registry.register(Box::new(active_rooms.clone()))?;

        let active_transports = IntGauge::with_opts(Opts::new(
            "media_active_transports",
            "Number of active WebRtcTransports",
        ))?;
        registry.register(Box::new(active_transports.clone()))?;

        let active_producers = IntGauge::with_opts(Opts::new(
            "media_active_producers",
            "Number of active producers",
        ))?;
        registry.register(Box::new(active_producers.clone()))?;

        let active_consumers = IntGauge::with_opts(Opts::new(
            "media_active_consumers",
            "Number of active consumers",
        ))?;
        registry.register(Box::new(active_consumers.clone()))?;

        let active_speakers = IntGauge::with_opts(Opts::new(
            "media_active_speakers",
            "Current aggregate speaker count across all rooms",
        ))?;
        registry.register(Box::new(active_speakers.clone()))?;
        let speaker_stream_dropped_updates = Counter::with_opts(Opts::new(
            "media_speaker_stream_dropped_updates_total",
            "Speaker snapshots skipped by slow stream subscribers",
        ))?;
        registry.register(Box::new(speaker_stream_dropped_updates.clone()))?;

        Ok(Self {
            active_workers,
            worker_failures,
            active_rooms,
            active_transports,
            active_producers,
            active_consumers,
            active_speakers,
            speaker_stream_dropped_updates,
        })
    }

    pub(crate) async fn set_all(&self, media: &MediaRuntime) {
        self.active_workers
            .set(saturating_i64(media.active_workers()));
        self.active_rooms
            .set(saturating_i64(media.room_count().await));
        self.active_transports
            .set(saturating_i64(media.transport_count().await));
        self.active_producers
            .set(saturating_i64(media.producer_count().await));
        self.active_consumers
            .set(saturating_i64(media.consumer_count().await));
        self.active_speakers
            .set(saturating_i64(media.speaker_hub().active_count()));
    }
}

pub(crate) fn spawn_supervision(
    health: HealthState,
    media: Arc<MediaRuntime>,
    metrics: MediaMetrics,
    mut shutdown: watch::Receiver<bool>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut last_active = media.active_workers();
        let expected = last_active;
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        interval.tick().await;
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let active = media.active_workers();
                    metrics.active_workers.set(saturating_i64(active));
                    metrics.active_rooms.set(saturating_i64(media.room_count().await));
                    metrics.active_transports.set(saturating_i64(media.transport_count().await));
                    metrics.active_producers.set(saturating_i64(media.producer_count().await));
                    metrics.active_consumers.set(saturating_i64(media.consumer_count().await));
                    metrics.active_speakers.set(saturating_i64(media.speaker_hub().active_count()));
                    if active < last_active {
                        let delta = last_active - active;
                        metrics.worker_failures.inc_by(f64::from(u32::try_from(delta).unwrap_or(0)));
                        tracing::warn!(active, expected, "mediasoup worker failure detected");
                    }
                    last_active = active;
                    health.set_check("mediasoup_workers", active > 0).await;
                }
                changed = shutdown.changed() => {
                    if changed.is_err() || *shutdown.borrow() {
                        break;
                    }
                }
            }
        }
    })
}
