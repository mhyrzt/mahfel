use crate::recording::spool::{directory_size, has_journals, remove_unjournaled};
use mahfel_application::{ports::RecordingStorage, recording_artifacts::FragmentReceipt};
use mahfel_config::RecordingSettings;
use mahfel_domain::RoomId;
use mahfel_rtc::{
    MediaRuntime,
    recording::{CaptureClock, CapturePipeline, RecordingTap},
};
use mahfel_rtc_protocol::media::{ReconcileRecordingRequest, ReconcileRecordingResponse};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
use tonic::Status;
use uuid::Uuid;

#[derive(Clone)]
pub struct Recordings {
    boot: Uuid,
    pub(super) metrics: crate::recording::metrics::RecordingMetrics,
    pub(super) upload_failed: Arc<AtomicBool>,
    media: Arc<MediaRuntime>,
    pub(super) storage: Arc<dyn RecordingStorage>,
    pub(super) settings: RecordingSettings,
    entries: Arc<Mutex<HashMap<(Uuid, u32), CaptureEntry>>>,
}
type CaptureEntry = Arc<Mutex<Capture>>;

struct Capture {
    request: ReconcileRecordingRequest,
    clock: CaptureClock,
    taps: HashMap<String, RecordingTap>,
    directory: PathBuf,
    report: ReconcileRecordingResponse,
    sealing: Option<Instant>,
    retry_at: Instant,
    failures: u32,
    keyframe_at: Instant,
    last_contact: Instant,
}

impl Recordings {
    /// Check the same room, spool and private object store used by capture
    /// before the control plane commits an auto-recorded session as live.
    pub async fn prepare(&self, room_id: &str) -> Result<(), Status> {
        let room = RoomId::from_uuid(
            Uuid::parse_str(room_id).map_err(|_| Status::invalid_argument("room id"))?,
        );
        if self.upload_failed.load(Ordering::Acquire) {
            return Err(Status::unavailable("recording upload unavailable"));
        }
        if self.entries.lock().await.len() >= 100 {
            return Err(Status::resource_exhausted("recording capacity"));
        }
        CapturePipeline::check_available()
            .map_err(|_| Status::unavailable("recording codecs unavailable"))?;
        let total = directory_size(self.settings.spool_directory.clone())
            .await
            .map_err(|_| Status::unavailable("recording spool unavailable"))?;
        if total >= self.settings.node_spool_bytes {
            return Err(Status::resource_exhausted("recording spool exhausted"));
        }
        tokio::fs::create_dir_all(&self.settings.spool_directory)
            .await
            .map_err(|_| Status::unavailable("recording spool unavailable"))?;
        let probe = self
            .settings
            .spool_directory
            .join(format!(".prepare-{}", Uuid::new_v4()));
        let spool_result = tokio::fs::write(&probe, b"recording-ready").await;
        let _ = tokio::fs::remove_file(&probe).await;
        spool_result.map_err(|_| Status::unavailable("recording spool unwritable"))?;
        let key = format!("recordings/preflight/{}", Uuid::new_v4());
        self.storage
            .put(&key, b"recording-ready".to_vec())
            .await
            .map_err(|_| Status::unavailable("recording storage unwritable"))?;
        let read = self.storage.read(&key, 32).await;
        let deleted = self.storage.delete(&key).await;
        if !matches!(read, Ok(ref bytes) if bytes == b"recording-ready") || deleted.is_err() {
            return Err(Status::unavailable("recording storage verification failed"));
        }
        self.media
            .create_room(room)
            .await
            .map_err(|_| Status::unavailable("media room unavailable"))?;
        Ok(())
    }

    pub fn new(
        media: Arc<MediaRuntime>,
        storage: Arc<dyn RecordingStorage>,
        settings: RecordingSettings,
        registry: &prometheus::Registry,
    ) -> Result<Self, Status> {
        if settings.upload_concurrency == 0
            || settings.upload_concurrency > 4
            || settings.recording_spool_bytes == 0
            || settings.node_spool_bytes < settings.recording_spool_bytes
        {
            return Err(Status::invalid_argument(
                "invalid recording spool configuration",
            ));
        }
        Ok(Self {
            boot: Uuid::new_v4(),
            metrics: crate::recording::metrics::RecordingMetrics::register(registry)
                .map_err(|_| Status::internal("recording metrics registration"))?,
            upload_failed: Arc::new(AtomicBool::new(false)),
            media,
            storage,
            settings,
            entries: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn reconcile(
        &self,
        request: ReconcileRecordingRequest,
    ) -> Result<ReconcileRecordingResponse, Status> {
        let id = Uuid::parse_str(&request.recording_id)
            .map_err(|_| Status::invalid_argument("recording id"))?;
        Uuid::parse_str(&request.room_id).map_err(|_| Status::invalid_argument("room id"))?;
        if request.generation == 0
            || request.revision == 0
            || !matches!(
                request.desired_state.as_str(),
                "recording" | "paused" | "stopping"
            )
        {
            return Err(Status::invalid_argument("invalid capture intent"));
        }
        let key = (id, request.generation);
        let mut entries = self.entries.lock().await;
        let mut expired = Vec::new();
        for (key, entry) in entries.iter() {
            let capture = entry.lock().await;
            if capture.report.sealed && capture.last_contact.elapsed() > Duration::from_secs(60) {
                expired.push(*key);
            }
        }
        for key in expired {
            entries.remove(&key);
        }
        let entry = if let Some(entry) = entries.get(&key) {
            Arc::clone(entry)
        } else {
            if entries.len() >= 100 {
                return Err(Status::resource_exhausted("recording capacity"));
            }
            // A generation is immutable. A newer one can never reactivate an older tap.
            for ((other, generation), entry) in entries.iter() {
                if *other == id && *generation > request.generation {
                    return Err(Status::failed_precondition("stale generation"));
                }
                if *other == id {
                    let mut previous = entry.lock().await;
                    previous.request.desired_state = "paused".into();
                }
            }
            let clock = CaptureClock::new()
                .map_err(|_| Status::unavailable("capture clock unavailable"))?;
            let directory = self
                .settings
                .spool_directory
                .join(id.to_string())
                .join(request.generation.to_string())
                .join(self.boot.to_string());
            let report = ReconcileRecordingResponse {
                boot_id: self.boot.to_string(),
                unix_origin_ns: clock.unix_origin_ns,
                capturing: false,
                sealed: false,
                warning: String::new(),
                receipt_prefix: format!(
                    "recordings/{id}/capture/{}/{}/receipts/",
                    request.generation, self.boot
                ),
            };
            let entry = Arc::new(Mutex::new(Capture {
                request: request.clone(),
                clock,
                taps: HashMap::new(),
                directory,
                report,
                sealing: None,
                retry_at: Instant::now(),
                failures: 0,
                keyframe_at: Instant::now(),
                last_contact: Instant::now(),
            }));
            self.metrics.active.inc();
            entries.insert(key, Arc::clone(&entry));
            let owner = self.clone();
            let task_entry = Arc::clone(&entry);
            tokio::spawn(async move {
                owner.capture_loop(task_entry).await;
            });
            entry
        };
        drop(entries);
        let mut capture = entry.lock().await;
        if request.room_id != capture.request.room_id {
            return Err(Status::failed_precondition("capture room is immutable"));
        }
        if request.revision < capture.request.revision {
            return Err(Status::failed_precondition("stale revision"));
        }
        if capture.report.sealed && request.desired_state == "recording" {
            return Err(Status::failed_precondition("generation sealed"));
        }
        capture.request = request;
        capture.last_contact = Instant::now();
        Ok(capture.report.clone())
    }

    async fn capture_loop(&self, entry: Arc<Mutex<Capture>>) {
        loop {
            let mut capture = entry.lock().await;
            if capture.report.sealed {
                self.metrics.active.dec();
                break;
            }
            if let Err(error) = self.tick(&mut capture).await {
                self.metrics
                    .failures
                    .with_label_values(&["capture_unavailable"])
                    .inc();
                tracing::warn!(%error, recording_id = %capture.request.recording_id, "recording capture interrupted");
                capture.report.warning = "capture_unavailable".into();
                capture.report.capturing = false;
                capture.failures = capture.failures.saturating_add(1);
                capture.retry_at = Instant::now()
                    + Duration::from_secs((1_u64 << capture.failures.min(5)).min(30));
                for tap in capture.taps.values() {
                    let _ = tap.seal().await;
                }
                capture.sealing.get_or_insert_with(Instant::now);
            }
            drop(capture);
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    }

    async fn tick(&self, capture: &mut Capture) -> anyhow::Result<()> {
        let room = RoomId::from_uuid(Uuid::parse_str(&capture.request.room_id)?);
        if self.upload_failed.load(Ordering::Acquire) {
            capture.report.warning = "upload_unavailable".into();
        }
        let own = directory_size(
            self.settings
                .spool_directory
                .join(&capture.request.recording_id),
        )
        .await?;
        let total = directory_size(self.settings.spool_directory.clone()).await?;
        self.metrics
            .spool
            .set(i64::try_from(total).unwrap_or(i64::MAX));
        let exhausted =
            own >= self.settings.recording_spool_bytes || total >= self.settings.node_spool_bytes;
        let running = capture.request.desired_state == "recording"
            && !exhausted
            && Instant::now() >= capture.retry_at;
        if exhausted {
            if capture.report.warning != "spool_exhausted" {
                self.metrics
                    .failures
                    .with_label_values(&["spool_exhausted"])
                    .inc();
            }
            capture.report.warning = "spool_exhausted".into();
            capture.report.capturing = false;
        }
        if !running && capture.sealing.is_none() {
            for tap in capture.taps.values() {
                tap.seal().await?;
            }
            capture.sealing = Some(Instant::now());
            capture.report.capturing = false;
        }
        if running && capture.sealing.is_none() {
            self.media.create_room(room).await?;
            for producer in self.media.list_producers(room).await {
                if !capture.taps.contains_key(&producer.producer_id) {
                    let directory = capture
                        .directory
                        .join(&producer.producer_id)
                        .join(Uuid::new_v4().to_string());
                    let tap = self
                        .media
                        .recording_tap(room, &producer.producer_id, &directory, &capture.clock)
                        .await?;
                    capture.taps.insert(producer.producer_id, tap);
                }
            }
            capture.report.capturing = true;
            if capture.report.warning != "upload_unavailable" {
                capture.report.warning.clear();
            }
            if capture.keyframe_at.elapsed() >= Duration::from_secs(5) {
                for tap in capture.taps.values() {
                    tap.request_key_frame().await?;
                }
                capture.keyframe_at = Instant::now();
            }
        }
        for tap in capture.taps.values() {
            if tap.closed() && !tap.pipeline.ended() {
                let _ = tap.pipeline.seal();
            }
            for fragment in tap.pipeline.drain()? {
                if tokio::fs::metadata(&fragment.path).await?.len() > 32 * 1024 * 1024 {
                    tokio::fs::remove_file(&fragment.path).await?;
                    anyhow::bail!("capture fragment exceeded bounded upload size");
                }
                let bytes = tokio::fs::read(&fragment.path).await?;
                if bytes.is_empty() {
                    continue;
                }
                let (hash, size) = tokio::task::spawn_blocking(move || {
                    (format!("{:x}", Sha256::digest(&bytes)), bytes.len())
                })
                .await?;
                let filename = fragment
                    .path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .ok_or_else(|| anyhow::anyhow!("invalid fragment path"))?;
                let instance = fragment
                    .path
                    .parent()
                    .and_then(|path| path.file_name())
                    .and_then(|name| name.to_str())
                    .ok_or_else(|| anyhow::anyhow!("invalid tap path"))?;
                let object_key = format!(
                    "recordings/{}/capture/{}/{}/tracks/{}/{instance}/{filename}",
                    capture.request.recording_id,
                    capture.request.generation,
                    self.boot,
                    tap.producer_id
                );
                let receipt = FragmentReceipt {
                    recording_id: Uuid::parse_str(&capture.request.recording_id)?,
                    generation: capture.request.generation,
                    boot_id: self.boot,
                    producer_id: tap.producer_id.clone(),
                    actor_id: tap.actor_id.as_uuid(),
                    source: tap.source.as_str().into(),
                    codec: tap.pipeline.codec.clone(),
                    container: tap.pipeline.container.clone(),
                    unix_origin_ns: capture.clock.unix_origin_ns,
                    start_ns: fragment.start_ns,
                    end_ns: fragment.end_ns,
                    object_key,
                    sha256: hash.clone(),
                    bytes: size as u64,
                };
                let journal = fragment.path.with_extension("receipt");
                let temporary = journal.with_extension("receipt-pending");
                tokio::fs::write(&temporary, serde_json::to_vec(&receipt)?).await?;
                tokio::fs::File::open(&temporary).await?.sync_all().await?;
                tokio::fs::rename(temporary, journal).await?;
            }
        }
        capture
            .taps
            .retain(|_, tap| !tap.pipeline.ended() || capture.sealing.is_some());
        if let Some(start) = capture.sealing {
            let ended = capture.taps.values().all(|tap| tap.pipeline.ended());
            if ended || start.elapsed() >= Duration::from_secs(25) {
                if !ended {
                    capture.report.warning = "seal_timeout".into();
                }
                capture.taps.clear();
                remove_unjournaled(capture.directory.clone()).await?;
                if capture.request.desired_state == "recording" {
                    capture.sealing = None;
                } else if !has_journals(capture.directory.clone()).await?
                    || start.elapsed() >= Duration::from_secs(25)
                {
                    let key = format!(
                        "{}sealed.json",
                        capture.report.receipt_prefix.trim_end_matches("receipts/")
                    );
                    if has_journals(capture.directory.clone()).await? {
                        capture.report.warning = "upload_incomplete".into();
                    }
                    let marker = serde_json::to_vec(
                        &serde_json::json!({ "boot_id": self.boot, "generation": capture.request.generation, "warning": capture.report.warning }),
                    )?;
                    if !matches!(
                        tokio::time::timeout(
                            Duration::from_secs(2),
                            self.storage.put(&key, marker)
                        )
                        .await,
                        Ok(Ok(()))
                    ) {
                        capture.report.warning = "upload_incomplete".into();
                    }
                    capture.report.sealed = true;
                }
            }
        }
        Ok(())
    }

    pub async fn seal_room(&self, room: &str) {
        let wait = async {
            let entries: Vec<_> = self.entries.lock().await.values().cloned().collect();
            let mut selected = Vec::new();
            for entry in entries {
                let mut capture = entry.lock().await;
                if capture.request.room_id == room && !capture.report.sealed {
                    capture.request.desired_state = "stopping".into();
                    selected.push(Arc::clone(&entry));
                }
            }
            loop {
                let mut sealed = true;
                for entry in &selected {
                    sealed &= entry.lock().await.report.sealed;
                }
                if sealed {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        };
        let _ = tokio::time::timeout(Duration::from_secs(30), wait).await;
    }

    pub async fn drain(&self) {
        let drain = async {
            let entries: Vec<_> = self.entries.lock().await.values().cloned().collect();
            for entry in &entries {
                entry.lock().await.request.desired_state = "stopping".into();
            }
            let wait = async {
                loop {
                    let mut sealed = true;
                    for entry in &entries {
                        sealed &= entry.lock().await.report.sealed;
                    }
                    if sealed {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(250)).await;
                }
            };
            wait.await;
        };
        let _ = tokio::time::timeout(Duration::from_secs(30), drain).await;
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
