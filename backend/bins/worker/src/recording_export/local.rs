use super::{
    decoder::{Decoder, crop_camera},
    encoder::Encoder,
};
use anyhow::{Context, Result, ensure};
use base64::Engine;
use serde_json::{Value, json};
use std::{collections::HashMap, path::Path, process::Stdio};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::Command,
};

struct Fragment {
    id: String,
    actor: String,
    source: String,
    path: std::path::PathBuf,
    start: i128,
    end: i128,
    audio: bool,
}
fn ns(value: &Value) -> Result<i128> {
    value
        .as_str()
        .context("timestamp")?
        .parse()
        .context("timestamp integer")
}

pub(crate) async fn render_local(directory: &Path, renderer: &Path) -> Result<()> {
    gstreamer::init()?;
    let input: Value =
        serde_json::from_slice(&tokio::fs::read(directory.join("input.json")).await?)?;
    let media: Vec<Value> =
        serde_json::from_slice(&tokio::fs::read(directory.join("media.json")).await?)?;
    let assets: Value =
        serde_json::from_slice(&tokio::fs::read(directory.join("assets.json")).await?)?;
    let settings: Value =
        serde_json::from_slice(&tokio::fs::read(directory.join("settings.json")).await?)?;
    let duration = u64::try_from(ns(&input["duration_ns"])?)?;
    let width = input["width"].as_i64().unwrap_or(1920) as i32;
    let height = input["height"].as_i64().unwrap_or(1080) as i32;
    ensure!(
        (16..=1920).contains(&width)
            && (16..=1080).contains(&height)
            && width % 2 == 0
            && height % 2 == 0,
        "invalid output dimensions"
    );
    let fragments = media
        .iter()
        .map(|item| {
            Ok(Fragment {
                id: item["id"].as_str().context("track id")?.into(),
                actor: item["actorId"].as_str().context("actor id")?.into(),
                source: item["source"].as_str().context("track source")?.into(),
                path: directory.join(item["path"].as_str().context("fragment path")?),
                start: ns(&item["start_ns"])?,
                end: ns(&item["end_ns"])?,
                audio: item["audio"].as_bool().context("track kind")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let mut track_starts = HashMap::<&str, i128>::new();
    for fragment in &fragments {
        track_starts
            .entry(&fragment.id)
            .and_modify(|start| *start = (*start).min(fragment.start))
            .or_insert(fragment.start);
    }
    let mut browser = Command::new(settings["bun"].as_str().unwrap_or("bun"))
        .arg(renderer.join("runner.ts"))
        .arg(directory)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()?;
    let mut writer = browser.stdin.take().context("renderer stdin")?;
    let mut reader = BufReader::new(browser.stdout.take().context("renderer stdout")?);
    request(
        &mut writer,
        &mut reader,
        &json!({ "action": "initialize", "input": input }),
    )
    .await?;
    let encoder = Encoder::new(&directory.join("export.mp4"), width, height)?;
    let mut decoders = HashMap::<usize, Decoder>::new();
    let mut presentation_video: Option<(String, Decoder, Option<Decoder>, u64)> = None;
    let frame_count = duration.div_ceil(1_000_000_000 / 30);
    for frame_index in 0..frame_count {
        let pts = frame_index * 1_000_000_000 / 30;
        if pts >= duration {
            break;
        }
        let next_pts = ((frame_index + 1) * 1_000_000_000 / 30).min(duration);
        let mut active = fragments
            .iter()
            .enumerate()
            .filter(|(_, fragment)| {
                fragment.start <= i128::from(pts) && i128::from(pts) < fragment.end
            })
            .collect::<Vec<_>>();
        active.sort_by_key(|(index, fragment)| (std::cmp::Reverse(fragment.start), *index));
        let mut seen = std::collections::HashSet::new();
        active.retain(|(_, fragment)| seen.insert(fragment.id.as_str()));
        decoders.retain(|index, _| active.iter().any(|(active, _)| active == index));
        let tracks = active.iter().filter(|(_, fragment)| !fragment.audio).map(|(_, fragment)| json!({ "id": fragment.id, "actorId": fragment.actor, "source": fragment.source, "started_ns": track_starts.get(fragment.id.as_str()).copied().unwrap_or(fragment.start).to_string() })).collect::<Vec<_>>();
        let response = request(&mut writer, &mut reader, &json!({ "action": "render", "timestamp": pts.to_string(), "tracks": tracks, "width": width, "height": height })).await?;
        let slate = !response["scene"]["slate"].is_null();
        let png = base64::engine::general_purpose::STANDARD
            .decode(response["png"].as_str().context("renderer frame")?)?;
        ensure!(
            png.len() <= 16 * 1024 * 1024,
            "renderer frame resource limit"
        );
        let mut videos = Vec::new();
        let audio_frames =
            usize::try_from(next_pts * 48000 / 1_000_000_000 - pts * 48000 / 1_000_000_000)?;
        let mut audio = vec![0.0f32; audio_frames * 2];
        if !slate {
            for layer in response["scene"]["layers"]
                .as_array()
                .context("scene layers")?
            {
                let id = layer["source"].as_str().context("scene source")?;
                if id == "workspace" {
                    continue;
                }
                if let Some((index, fragment)) =
                    active.iter().find(|(_, fragment)| fragment.id == id)
                {
                    if !decoders.contains_key(index) {
                        decoders.insert(*index, Decoder::open(&fragment.path, false)?);
                    }
                    if let Some(mut frame) = decoders
                        .get_mut(index)
                        .context("video decoder")?
                        .video(u64::try_from(i128::from(pts) - fragment.start)?)?
                    {
                        if layer["fit"] == "cover" {
                            frame = crop_camera(
                                frame,
                                i32::try_from(layer["width"].as_i64().context("tile width")?)?,
                                i32::try_from(layer["height"].as_i64().context("tile height")?)?,
                            )?;
                        }
                        videos.push((frame, layer.clone()));
                    }
                }
            }
            let microphones = active
                .iter()
                .filter(|(_, fragment)| fragment.audio)
                .collect::<Vec<_>>();
            ensure!(
                microphones.len()
                    <= settings["max_active_audio_tracks"].as_u64().unwrap_or(64) as usize,
                "audio resource limit"
            );
            for (index, fragment) in microphones {
                if !decoders.contains_key(index) {
                    decoders.insert(*index, Decoder::open(&fragment.path, true)?);
                }
                let decoded = decoders.get_mut(index).context("audio decoder")?.audio(
                    u64::try_from(i128::from(pts) - fragment.start)?,
                    audio_frames,
                    1.0,
                )?;
                for (mixed, value) in audio.iter_mut().zip(decoded) {
                    *mixed += value;
                }
            }
            let presented = &response["presentationVideo"];
            if let Some(id) = presented["file_id"].as_str() {
                let position = (presented["position_ms"]
                    .as_f64()
                    .context("presentation position")?
                    * 1_000_000.0) as u64;
                if presentation_video
                    .as_ref()
                    .is_none_or(|(current, _, _, _)| current != id)
                {
                    let path =
                        directory.join(assets[id]["path"].as_str().context("presentation file")?);
                    let video = Decoder::open(&path, false)?;
                    let audio = Some(Decoder::optional_audio(&path)?);
                    presentation_video = Some((id.into(), video, audio, 0));
                }
                let (_, video, asset_audio, previous) = presentation_video
                    .as_mut()
                    .context("presentation decoder")?;
                if position < *previous || position.abs_diff(*previous) > 500_000_000 {
                    video.seek(position)?;
                    if let Some(decoder) = asset_audio {
                        decoder.seek(position)?;
                    }
                }
                if let Some(frame) = video.video(position)? {
                    videos.push((
                        frame,
                        json!({ "x": 0, "y": 0, "width": width, "height": height * 5 / 6 }),
                    ));
                }
                if presented["paused"] == false
                    && let Some(decoder) = asset_audio
                {
                    let values = decoder.audio(
                        position,
                        audio_frames,
                        presented["rate"].as_f64().unwrap_or(1.0),
                    )?;
                    for (mixed, value) in audio.iter_mut().zip(values) {
                        *mixed += value;
                    }
                }
                *previous = position;
            } else {
                presentation_video = None;
            }
        }
        for value in &mut audio {
            *value = if value.is_finite() {
                value.clamp(-1.0, 1.0)
            } else {
                0.0
            };
        }
        encoder.frame(png, videos, audio, pts, next_pts - pts)?;
        if frame_index % 30 == 0 {
            let size = std::fs::metadata(directory.join("export.mp4"))?.len().max(
                std::fs::metadata(directory.join("export.mux.tmp"))
                    .map(|metadata| metadata.len())
                    .unwrap_or(0),
            );
            ensure!(
                size <= settings["output_bytes"]
                    .as_u64()
                    .context("output resource limit")?,
                "output resource limit"
            );
            tokio::fs::write(
                directory.join("progress.tmp"),
                (frame_index * 99 / frame_count.max(1)).to_string(),
            )
            .await?;
            tokio::fs::rename(directory.join("progress.tmp"), directory.join("progress")).await?;
        }
    }
    encoder.finish()?;
    writer.write_all(b"{\"action\":\"dispose\"}\n").await?;
    writer.shutdown().await?;
    drop(writer);
    tokio::time::timeout(std::time::Duration::from_secs(10), browser.wait()).await??;
    super::verification::verify(&directory.join("export.mp4"), duration)?;
    Ok(())
}

async fn request(
    writer: &mut tokio::process::ChildStdin,
    reader: &mut BufReader<tokio::process::ChildStdout>,
    request: &Value,
) -> Result<Value> {
    let mut bytes = serde_json::to_vec(request)?;
    bytes.push(b'\n');
    writer.write_all(&bytes).await?;
    writer.flush().await?;
    let mut response = Vec::new();
    let count = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        (&mut *reader)
            .take(32 * 1024 * 1024)
            .read_until(b'\n', &mut response),
    )
    .await??;
    ensure!(
        count > 0 && response.last() == Some(&b'\n'),
        "renderer response unavailable or too large"
    );
    serde_json::from_slice(&response).context("renderer response")
}
