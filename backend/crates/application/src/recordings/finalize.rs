use crate::{
    jobs::{JobEnvelope, JobFailure},
    ports::{RecordingFinalizationStore, RecordingStorage, RecordingVerifier},
    recordings::artifacts::FragmentReceipt,
};
use base64::Engine;
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Deserialize)]
struct Payload {
    recording_id: Uuid,
    capture_generation: i32,
}

pub async fn finalize_recording<
    R: RecordingFinalizationStore + ?Sized,
    S: RecordingStorage + ?Sized,
    V: RecordingVerifier + ?Sized,
>(
    records: &R,
    storage: &S,
    verifier: &V,
    job: &JobEnvelope,
    token: Uuid,
) -> Result<(), JobFailure> {
    let payload: Payload = serde_json::from_value(job.payload.clone())
        .map_err(|_| JobFailure::Permanent("invalid_payload"))?;
    let Some(metadata) = records
        .metadata(payload.recording_id, payload.capture_generation)
        .await
        .map_err(|_| JobFailure::Retryable("recording_store_unavailable"))?
    else {
        return Err(JobFailure::Permanent("recording_generation_unsealed"));
    };
    if metadata["status"] == "ready" || metadata["status"] == "failed" {
        return Ok(());
    }
    let mut partial = metadata["partial_coverage"].as_bool().unwrap_or(false);
    let mut usable = false;
    let mut sections = BTreeMap::new();
    let mut tracks = BTreeMap::new();
    let mut browser_segments = Vec::new();
    let session_origin: i128 = metadata["session_origin_ns"]
        .as_str()
        .ok_or(JobFailure::Permanent("invalid_session_clock"))?
        .parse()
        .map_err(|_| JobFailure::Permanent("invalid_session_clock"))?;
    let mut losses = Vec::new();
    let mut intervals = Vec::new();
    let mut coverage = BTreeMap::<String, Vec<(u64, u64)>>::new();
    for section in [
        "generations",
        "snapshots",
        "events",
        "assets",
        "fragments",
        "actors",
    ] {
        let mut offset = 0;
        let mut parts = Vec::new();
        loop {
            let mut page = records
                .page(payload.recording_id, section, offset)
                .await
                .map_err(|_| JobFailure::Retryable("recording_store_unavailable"))?;
            let count = page.len();
            if count == 0 {
                break;
            }
            if section == "generations" {
                intervals.extend(page.iter().cloned());
                for generation in &page {
                    if let Some(prefix) = generation["receipt_prefix"].as_str() {
                        let mut continuation = None;
                        loop {
                            let receipts =
                                storage.page(prefix, continuation).await.map_err(|_| {
                                    JobFailure::Retryable("receipt_storage_unavailable")
                                })?;
                            for key in receipts.keys {
                                let bytes = storage.read(&key, 8192).await.map_err(|_| {
                                    JobFailure::Retryable("receipt_storage_unavailable")
                                })?;
                                let receipt: FragmentReceipt = serde_json::from_slice(&bytes)
                                    .map_err(|_| {
                                        JobFailure::Permanent("invalid_fragment_receipt")
                                    })?;
                                if receipt.recording_id == payload.recording_id {
                                    records.import_receipt(&receipt).await.map_err(|_| {
                                        JobFailure::Retryable("recording_store_unavailable")
                                    })?;
                                }
                            }
                            continuation = receipts.continuation;
                            if continuation.is_none() {
                                break;
                            }
                        }
                    }
                    if !generation["loss_reason"].is_null() {
                        partial = true;
                    }
                }
            } else if section == "fragments" {
                let mut valid = Vec::new();
                for value in page {
                    let receipt: FragmentReceipt = serde_json::from_value(value)
                        .map_err(|_| JobFailure::Permanent("invalid_fragment_receipt"))?;
                    let bytes = storage
                        .read(&receipt.object_key, 32 * 1024 * 1024)
                        .await
                        .map_err(|_| JobFailure::Retryable("fragment_storage_unavailable"))?;
                    let (checksum, bytes) = tokio::task::spawn_blocking(move || {
                        (format!("{:x}", Sha256::digest(&bytes)), bytes)
                    })
                    .await
                    .map_err(|_| JobFailure::Retryable("checksum_verifier_unavailable"))?;
                    let readable = checksum == receipt.sha256
                        && bytes.len() as u64 == receipt.bytes
                        && verifier
                            .readable(bytes.clone(), &receipt.container)
                            .await
                            .map_err(|_| JobFailure::Retryable("container_verifier_unavailable"))?;
                    if readable {
                        if browser_segments.len() >= super::playback::MAX_MEDIA_SEGMENTS {
                            return Err(JobFailure::Permanent("playback_segment_limit"));
                        }
                        let browser = verifier
                            .browser_segment(
                                bytes,
                                &receipt.container,
                                receipt.codec.starts_with("audio/"),
                            )
                            .await
                            .map_err(|error| {
                                tracing::warn!(%error, recording_id = %payload.recording_id, "browser segment conversion failed");
                                JobFailure::Retryable("browser_segment_unavailable")
                            })?;
                        let browser_hash = format!("{:x}", Sha256::digest(&browser));
                        let browser_key = format!(
                            "recordings/{}/replay/{}/media/{browser_hash}.mp4",
                            payload.recording_id, payload.capture_generation
                        );
                        storage
                            .put_media(&browser_key, browser)
                            .await
                            .map_err(|_| {
                                JobFailure::Retryable("browser_segment_storage_unavailable")
                            })?;
                        let start = i128::from(receipt.unix_origin_ns)
                            + i128::from(receipt.start_ns)
                            - session_origin;
                        let end = i128::from(receipt.unix_origin_ns) + i128::from(receipt.end_ns)
                            - session_origin;
                        browser_segments.push(json!({
                            "track_id": format!("{}:{}:{}", receipt.generation, receipt.boot_id, receipt.producer_id),
                            "actor_id": receipt.actor_id,
                            "source": receipt.source,
                            "audio": receipt.codec.starts_with("audio/"),
                            "start_ns": start.max(0).to_string(),
                            "end_ns": end.max(0).to_string(),
                            "object_key": browser_key,
                            "sha256": browser_hash,
                        }));
                        usable = true;
                        coverage
                            .entry(format!(
                                "{}:{}:{}",
                                receipt.generation, receipt.boot_id, receipt.producer_id
                            ))
                            .or_default()
                            .push((receipt.start_ns, receipt.end_ns));
                        tracks.entry(format!("{}:{}:{}", receipt.generation, receipt.boot_id, receipt.producer_id)).or_insert_with(|| json!({
                            "producer_id": receipt.producer_id, "actor_id": receipt.actor_id, "source": receipt.source,
                            "codec": receipt.codec, "container": receipt.container, "generation": receipt.generation,
                            "boot_id": receipt.boot_id, "unix_origin_ns": receipt.unix_origin_ns,
                        }));
                        valid.push(
                            serde_json::to_value(receipt)
                                .map_err(|_| JobFailure::Permanent("manifest_serialization"))?,
                        );
                    } else {
                        partial = true;
                        losses.push(json!({"reason":"invalid_fragment", "object_key":receipt.object_key, "unix_origin_ns":receipt.unix_origin_ns, "start_ns":receipt.start_ns, "end_ns":receipt.end_ns}));
                    }
                }
                page = valid;
            } else if section == "assets" {
                for asset in &mut page {
                    let destination = asset["archive_key"]
                        .as_str()
                        .ok_or(JobFailure::Permanent("invalid_asset"))?
                        .to_owned();
                    if asset["copy_status"] != "ready" {
                        let source = asset["source_key"]
                            .as_str()
                            .ok_or(JobFailure::Permanent("invalid_asset"))?;
                        storage
                            .copy(source, &destination)
                            .await
                            .map_err(|_| JobFailure::Retryable("asset_copy_unavailable"))?;
                    }
                    let bytes = storage
                        .read(&destination, 50 * 1024 * 1024)
                        .await
                        .map_err(|_| JobFailure::Retryable("asset_storage_unavailable"))?;
                    let (checksum, size) = tokio::task::spawn_blocking(move || {
                        (
                            base64::engine::general_purpose::STANDARD
                                .encode(Sha256::digest(&bytes)),
                            bytes.len(),
                        )
                    })
                    .await
                    .map_err(|_| JobFailure::Retryable("checksum_verifier_unavailable"))?;
                    if checksum != asset["checksum_sha_256"]
                        || asset["size_bytes"].as_u64() != Some(size as u64)
                    {
                        return Err(JobFailure::Retryable("asset_checksum_mismatch"));
                    }
                    asset["copy_status"] = json!("ready");
                    asset
                        .as_object_mut()
                        .map(|object| object.remove("source_key"));
                }
            } else if section != "actors" {
                usable = true;
            }
            let bytes = serde_json::to_vec(&page)
                .map_err(|_| JobFailure::Permanent("manifest_serialization"))?;
            let hash = format!("{:x}", Sha256::digest(&bytes));
            let key = format!(
                "recordings/{}/replay/{}/{section}/{offset}-{hash}.json",
                payload.recording_id, payload.capture_generation
            );
            storage
                .put(&key, bytes)
                .await
                .map_err(|_| JobFailure::Retryable("manifest_storage_unavailable"))?;
            parts.push(json!({"object_key":key, "sha256":hash, "entries":page.len()}));
            offset += i64::try_from(count).map_err(|_| JobFailure::Permanent("manifest_limit"))?;
            if count < 100 {
                break;
            }
        }
        sections.insert(section, parts);
    }
    // Source timestamps and generation intervals preserve pauses and outages on the session clock.
    let mut gaps = Vec::new();
    for interval in &intervals {
        if !interval["loss_reason"].is_null() {
            gaps.push(json!({"reason":"capture_coverage_uncertain", "generation":interval["generation"], "loss_reason":interval["loss_reason"], "start_ns":interval["start_ns"], "end_ns":interval["end_ns"]}));
        }
    }
    for pair in intervals.windows(2) {
        if !pair[0]["ended_at"].is_null() {
            gaps.push(json!({"reason": if pair[0]["loss_reason"].is_null() { "paused" } else { "capture_outage" }, "start_at":pair[0]["ended_at"], "end_at":pair[1]["started_at"]}));
        }
    }
    for (track, mut bounds) in coverage {
        bounds.sort_unstable();
        for pair in bounds.windows(2) {
            if pair[1].0 > pair[0].1.saturating_add(100_000_000) {
                gaps.push(json!({"reason":"media_gap", "track":track, "start_ns":pair[0].1, "end_ns":pair[1].0}));
            }
        }
    }
    let manifest = json!({"version":1, "recording":metadata, "partial":partial, "clock":"session_relative_nanoseconds", "bytea_encoding":"postgres_hex", "tracks":tracks, "sections":sections, "intervals":intervals, "gaps":gaps, "losses":losses});
    let bytes = serde_json::to_vec(&manifest)
        .map_err(|_| JobFailure::Permanent("manifest_serialization"))?;
    let hash = format!("{:x}", Sha256::digest(&bytes));
    let key = format!(
        "recordings/{}/replay/{}/manifest-{hash}.json",
        payload.recording_id, payload.capture_generation
    );
    storage
        .put(&key, bytes)
        .await
        .map_err(|_| JobFailure::Retryable("manifest_storage_unavailable"))?;
    browser_segments.sort_by(|left, right| {
        let start = |value: &serde_json::Value| {
            value["start_ns"]
                .as_str()
                .and_then(|value| value.parse::<i128>().ok())
                .unwrap_or(0)
        };
        start(left).cmp(&start(right))
    });
    let playback = json!({
        "version": 1,
        "clock": "session_relative_nanoseconds",
        "segments": browser_segments,
    });
    let playback_bytes = serde_json::to_vec(&playback)
        .map_err(|_| JobFailure::Permanent("playback_index_serialization"))?;
    if playback_bytes.len() as u64 > super::playback::INDEX_MAX_BYTES {
        return Err(JobFailure::Permanent("playback_index_limit"));
    }
    let playback_hash = format!("{:x}", Sha256::digest(&playback_bytes));
    let playback_key = format!(
        "recordings/{}/replay/{}/playback-{playback_hash}.json",
        payload.recording_id, payload.capture_generation
    );
    storage
        .put(&playback_key, playback_bytes)
        .await
        .map_err(|_| JobFailure::Retryable("playback_index_storage_unavailable"))?;
    if !records
        .accept(
            job,
            token,
            &crate::ports::ManifestAcceptance {
                recording: payload.recording_id,
                generation: payload.capture_generation,
                key,
                playback_key,
                partial,
                usable,
            },
        )
        .await
        .map_err(|_| JobFailure::Retryable("recording_commit_unavailable"))?
    {
        return Err(JobFailure::Retryable("recording_claim_lost"));
    }
    Ok(())
}
