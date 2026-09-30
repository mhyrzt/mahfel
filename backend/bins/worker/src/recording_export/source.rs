use anyhow::{Context, Result, ensure};
use mahfel_application::ports::{ExportSource, RecordingStorage};
use mahfel_config::RecordingExportSettings;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::Path;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

fn ns(value: &Value) -> Result<i128> {
    value
        .as_str()
        .context("timestamp string")?
        .parse()
        .context("timestamp integer")
}
fn time(value: &Value) -> Result<i128> {
    Ok(
        OffsetDateTime::parse(value.as_str().context("recording time")?, &Rfc3339)?
            .unix_timestamp_nanos(),
    )
}
fn key<'a>(value: &'a Value, prefix: &str) -> Result<&'a str> {
    let key = value.as_str().context("object key")?;
    ensure!(
        key.starts_with(prefix) && !key.contains(".."),
        "object outside recording"
    );
    Ok(key)
}
fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) async fn prepare(
    storage: &dyn RecordingStorage,
    source: &ExportSource,
    settings: &RecordingExportSettings,
    directory: &Path,
) -> Result<()> {
    let prefix = format!("recordings/{}/", source.recording_id);
    ensure!(
        source.manifest_key.starts_with(&prefix),
        "manifest outside recording"
    );
    let bytes = storage.read(&source.manifest_key, 8 * 1024 * 1024).await?;
    let expected = source
        .manifest_key
        .rsplit('/')
        .next()
        .and_then(|name| name.strip_prefix("manifest-"))
        .and_then(|name| name.strip_suffix(".json"))
        .context("manifest checksum")?;
    ensure!(sha(&bytes) == expected, "manifest checksum mismatch");
    let manifest: Value = serde_json::from_slice(&bytes)?;
    ensure!(
        manifest["version"] == 1 && manifest["clock"] == "session_relative_nanoseconds",
        "unsupported recording manifest"
    );
    let recording = &manifest["recording"];
    ensure!(
        recording["id"].as_str() == Some(&source.recording_id.to_string()),
        "recording identity mismatch"
    );
    let epoch = time(&recording["started_at"])?;
    let stop = time(&recording["stopped_at"])?;
    ensure!(stop >= epoch, "invalid recording bounds");
    let origin = ns(&recording["session_origin_ns"])?;
    let mut sections = std::collections::BTreeMap::<String, Vec<Value>>::new();
    let mut metadata_bytes = bytes.len();
    for (section, pages) in manifest["sections"]
        .as_object()
        .context("manifest sections")?
    {
        let mut values = Vec::new();
        for page in pages.as_array().context("section pages")? {
            let bytes = storage
                .read(key(&page["object_key"], &prefix)?, 8 * 1024 * 1024)
                .await?;
            metadata_bytes += bytes.len();
            ensure!(
                metadata_bytes <= 64 * 1024 * 1024,
                "replay metadata resource limit"
            );
            ensure!(
                sha(&bytes) == page["sha256"].as_str().context("section checksum")?,
                "section checksum mismatch"
            );
            let entries: Vec<Value> = serde_json::from_slice(&bytes)?;
            ensure!(
                entries.len() as u64 == page["entries"].as_u64().context("entry count")?,
                "section count mismatch"
            );
            values.extend(entries);
        }
        ensure!(values.len() <= 100000, "section entry limit");
        sections.insert(section.clone(), values);
    }
    let mut staged_bytes = metadata_bytes as u64;
    let mut media = Vec::new();
    for (index, receipt) in sections.get("fragments").into_iter().flatten().enumerate() {
        let object = key(&receipt["object_key"], &prefix)?;
        let container = receipt["container"]
            .as_str()
            .context("fragment container")?;
        ensure!(
            matches!(container, "webm" | "mkv"),
            "unsupported fragment container"
        );
        let relative = format!("fragment-{index}.{container}");
        let path = directory.join(&relative);
        let count = storage.download(object, &path, 32 * 1024 * 1024).await?;
        staged_bytes += count;
        ensure!(
            staged_bytes
                < settings
                    .scratch_bytes
                    .saturating_sub(settings.output_bytes.saturating_mul(2)),
            "scratch resource limit"
        );
        let bytes = tokio::fs::read(&path).await?;
        ensure!(
            count == receipt["bytes"].as_u64().context("fragment length")?
                && sha(&bytes) == receipt["sha256"].as_str().context("fragment checksum")?,
            "fragment checksum mismatch"
        );
        let clock = receipt["unix_origin_ns"]
            .as_u64()
            .context("fragment clock")? as i128;
        let start = clock + receipt["start_ns"].as_u64().context("fragment start")? as i128 - epoch;
        let end = clock + receipt["end_ns"].as_u64().context("fragment end")? as i128 - epoch;
        ensure!(end >= start, "invalid fragment bounds");
        media.push(json!({ "path": relative, "id": format!("{}:{}:{}", receipt["generation"], receipt["boot_id"].as_str().context("fragment boot")?, receipt["producer_id"].as_str().context("producer")?), "actorId": receipt["actor_id"], "source": receipt["source"], "start_ns": start.to_string(), "end_ns": end.to_string(), "audio": receipt["source"] == "microphone" }));
    }
    let mut assets = serde_json::Map::new();
    for asset in sections.get("assets").into_iter().flatten() {
        use base64::Engine;
        let id = uuid::Uuid::parse_str(asset["file_id"].as_str().context("asset id")?)?;
        let relative = format!("asset-{id}");
        let path = directory.join(&relative);
        let count = storage
            .download(
                key(&asset["archive_key"], &prefix)?,
                &path,
                50 * 1024 * 1024,
            )
            .await?;
        staged_bytes += count;
        ensure!(
            staged_bytes
                < settings
                    .scratch_bytes
                    .saturating_sub(settings.output_bytes.saturating_mul(2)),
            "scratch resource limit"
        );
        let bytes = tokio::fs::read(&path).await?;
        let checksum = base64::engine::general_purpose::STANDARD.encode(Sha256::digest(&bytes));
        ensure!(
            count == asset["size_bytes"].as_u64().context("asset size")?
                && checksum
                    == asset["checksum_sha_256"]
                        .as_str()
                        .context("asset checksum")?,
            "asset checksum mismatch"
        );
        assets.insert(
            id.to_string(),
            json!({ "path": relative, "mime": asset["mime"] }),
        );
    }
    let actors = sections
        .get("actors")
        .into_iter()
        .flatten()
        .filter_map(|actor| {
            Some((
                actor["actor_id"].as_str()?.to_owned(),
                actor["display_name"].clone(),
            ))
        })
        .collect::<serde_json::Map<String, Value>>();
    let input = json!({ "version": 1, "partial": manifest["partial"], "start_ns": (epoch - origin).to_string(), "duration_ns": (stop - epoch).to_string(), "snapshots": sections.get("snapshots").cloned().unwrap_or_default(), "events": sections.get("events").cloned().unwrap_or_default(), "intervals": manifest["intervals"], "assets": sections.get("assets").cloned().unwrap_or_default(), "actors": actors });
    tokio::fs::write(directory.join("input.json"), serde_json::to_vec(&input)?).await?;
    tokio::fs::write(directory.join("media.json"), serde_json::to_vec(&media)?).await?;
    tokio::fs::write(directory.join("assets.json"), serde_json::to_vec(&assets)?).await?;
    tokio::fs::write(directory.join("settings.json"), json!({ "output_bytes": settings.output_bytes, "max_active_audio_tracks": settings.max_active_audio_tracks, "bun": settings.bun }).to_string()).await?;
    Ok(())
}
