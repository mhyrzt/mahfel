use mahfel_application::{
    jobs::JobFailure,
    ports::{
        BoxFuture, ExportArtifact, ExportSource, FileStorageError, RecordingExporter,
        RecordingStorage,
    },
};
use mahfel_config::RecordingExportSettings;
use mahfel_postgres::PostgresRecordingStore;
use mahfel_storage::S3FileStorage;
use sha2::{Digest, Sha256};
use std::{path::PathBuf, process::Stdio};
use tokio::{io::AsyncReadExt, process::Command};
use uuid::Uuid;

pub(crate) struct GstreamerExporter {
    pub storage: S3FileStorage,
    pub store: PostgresRecordingStore,
    pub settings: RecordingExportSettings,
}
struct Workspace(PathBuf);
impl Drop for Workspace {
    fn drop(&mut self) {
        let directory = self.0.clone();
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            runtime.spawn_blocking(move || {
                let _ = std::fs::remove_dir_all(directory);
            });
        } else {
            let _ = std::fs::remove_dir_all(directory);
        }
    }
}
struct ProcessGroup(u32);
impl Drop for ProcessGroup {
    fn drop(&mut self) {
        let _ = std::process::Command::new("kill")
            .args(["-TERM", "--", &format!("-{}", self.0)])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

impl RecordingExporter for GstreamerExporter {
    fn discard<'a>(
        &'a self,
        artifact: &'a ExportArtifact,
    ) -> BoxFuture<'a, Result<(), JobFailure>> {
        Box::pin(async move {
            mahfel_application::ports::FileStorage::delete(&self.storage, &artifact.object_key)
                .await
                .map_err(|_| JobFailure::Retryable("export_cleanup_unavailable"))
        })
    }

    fn export<'a>(
        &'a self,
        source: &'a ExportSource,
        token: Uuid,
    ) -> BoxFuture<'a, Result<ExportArtifact, JobFailure>> {
        Box::pin(async move {
            let export = async {
                let directory = self
                    .settings
                    .scratch_directory
                    .join(format!("{}-{token}", source.export_id));
                tokio::fs::create_dir_all(&directory)
                    .await
                    .map_err(|_| JobFailure::Retryable("export_scratch_unavailable"))?;
                let workspace = Workspace(directory);
                super::source::prepare(&self.storage, source, &self.settings, &workspace.0)
                    .await
                    .map_err(|error| {
                        tracing::warn!(%error, "recording export input preparation failed");
                        match error.downcast_ref::<FileStorageError>() {
                            Some(FileStorageError::Unavailable) => {
                                JobFailure::Retryable("export_input_unavailable")
                            }
                            Some(FileStorageError::TooLarge) => {
                                JobFailure::Permanent("export_resource_limit")
                            }
                            Some(FileStorageError::NotFound) => {
                                JobFailure::Permanent("export_input_missing")
                            }
                            None if error.downcast_ref::<std::io::Error>().is_some() => {
                                JobFailure::Retryable("export_scratch_unavailable")
                            }
                            None => JobFailure::Permanent("export_input_invalid"),
                        }
                    })?;
                let renderer = std::fs::canonicalize(&self.settings.renderer_directory)
                    .map_err(|_| JobFailure::Permanent("export_renderer_missing"))?;
                let mut child = Command::new(
                    std::env::current_exe()
                        .map_err(|_| JobFailure::Permanent("export_worker_path"))?,
                )
                .arg("render-export")
                .arg(&workspace.0)
                .arg(&renderer)
                .process_group(0)
                .kill_on_drop(true)
                .stdout(Stdio::null())
                .stderr(Stdio::inherit())
                .spawn()
                .map_err(|_| JobFailure::Retryable("export_process_unavailable"))?;
                let _group = ProcessGroup(
                    child
                        .id()
                        .ok_or(JobFailure::Retryable("export_process_unavailable"))?,
                );
                let deadline = tokio::time::sleep(std::time::Duration::from_secs(
                    self.settings.timeout_seconds,
                ));
                tokio::pin!(deadline);
                let mut poll = tokio::time::interval(std::time::Duration::from_secs(2));
                loop {
                    tokio::select! {
                        result = child.wait() => {
                            if !result.map_err(|_| JobFailure::Retryable("export_process_failed"))?.success() { return Err(JobFailure::Permanent("export_render_failed")); }
                            break;
                        }
                        _ = &mut deadline => return Err(JobFailure::Permanent("export_timeout")),
                        _ = poll.tick() => {
                            let progress = tokio::fs::read_to_string(workspace.0.join("progress")).await.ok().and_then(|value| value.parse::<i32>().ok()).unwrap_or(0);
                            if !self.store.export_progress(source.export_id, token, progress).await.map_err(|_| JobFailure::Retryable("export_progress_unavailable"))? { return Err(JobFailure::Retryable("export_claim_lost")); }
                        }
                    }
                }
                let path = workspace.0.join("export.mp4");
                let size = tokio::fs::metadata(&path)
                    .await
                    .map_err(|_| JobFailure::Retryable("export_output_missing"))?
                    .len();
                if size == 0 || size > self.settings.output_bytes {
                    return Err(JobFailure::Permanent("export_output_resource_limit"));
                }
                let mut file = tokio::fs::File::open(&path)
                    .await
                    .map_err(|_| JobFailure::Retryable("export_output_missing"))?;
                let mut hasher = Sha256::new();
                let mut buffer = vec![0; 1024 * 1024];
                loop {
                    let count = file
                        .read(&mut buffer)
                        .await
                        .map_err(|_| JobFailure::Retryable("export_output_unreadable"))?;
                    if count == 0 {
                        break;
                    }
                    hasher.update(&buffer[..count]);
                }
                let input: serde_json::Value = serde_json::from_slice(
                    &tokio::fs::read(workspace.0.join("input.json"))
                        .await
                        .map_err(|_| JobFailure::Retryable("export_input_missing"))?,
                )
                .map_err(|_| JobFailure::Permanent("export_input_invalid"))?;
                let duration_ns = input["duration_ns"]
                    .as_str()
                    .and_then(|value| value.parse::<u64>().ok())
                    .ok_or(JobFailure::Permanent("export_duration_invalid"))?;
                let object_key = format!(
                    "recordings/{}/exports/{}/{}-{token}.mp4",
                    source.recording_id, source.export_id, source.revision
                );
                self.storage
                    .upload_export(&object_key, &path)
                    .await
                    .map_err(|_| JobFailure::Retryable("export_upload_unavailable"))?;
                Ok(ExportArtifact {
                    object_key,
                    duration_ms: i64::try_from(duration_ns / 1_000_000)
                        .map_err(|_| JobFailure::Permanent("export_duration_invalid"))?,
                    size_bytes: i64::try_from(size)
                        .map_err(|_| JobFailure::Permanent("export_size_invalid"))?,
                    sha256: format!("{:x}", hasher.finalize()),
                })
            };
            tokio::time::timeout(
                std::time::Duration::from_secs(self.settings.timeout_seconds),
                export,
            )
            .await
            .map_err(|_| JobFailure::Permanent("export_timeout"))?
        })
    }
}
