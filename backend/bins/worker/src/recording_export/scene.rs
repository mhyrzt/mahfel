use super::replay::Replay;
use serde_json::{Value, json};

pub(super) fn describe(
    state: &Replay,
    pts: u64,
    tracks: &[Value],
    width: i32,
    height: i32,
) -> Value {
    let mut tracks = tracks.to_vec();
    tracks.sort_by(|a, b| {
        a["actorId"]
            .as_str()
            .cmp(&b["actorId"].as_str())
            .then_with(|| a["id"].as_str().cmp(&b["id"].as_str()))
    });
    let screen = tracks
        .iter()
        .filter(|track| track["source"] == "screen")
        .max_by(|a, b| {
            a["started_ns"]
                .as_str()
                .and_then(|value| value.parse::<i128>().ok())
                .unwrap_or_default()
                .cmp(
                    &b["started_ns"]
                        .as_str()
                        .and_then(|value| value.parse::<i128>().ok())
                        .unwrap_or_default(),
                )
                .then_with(|| a["id"].as_str().cmp(&b["id"].as_str()))
        });
    let content = screen.is_some() || state.surface.is_some();
    let cameras = tracks
        .iter()
        .filter(|track| track["source"] == "camera")
        .collect::<Vec<_>>();
    let per_page = if content { 6 } else { 9 };
    let pages = cameras.len().div_ceil(per_page).max(1);
    let page = (pts / 10_000_000_000) as usize % pages;
    let mut layers = Vec::new();
    if state.slate.is_none() {
        if let Some(screen) = screen {
            layers.push(json!({"source":screen["id"],"x":0,"y":0,"width":width,"height":height*5/6,"fit":"contain","z":1}));
        } else if content {
            layers.push(json!({"source":"workspace","x":0,"y":0,"width":width,"height":height*5/6,"fit":"contain","z":2}));
        }
        let columns = if content { 6 } else { 3 };
        for (index, track) in cameras
            .iter()
            .skip(page * per_page)
            .take(per_page)
            .enumerate()
        {
            layers.push(json!({"source":track["id"],"x":index%columns*width as usize/columns,"y": if content {height as usize*5/6} else {index/columns*height as usize/3},"width":width as usize/columns,"height":height/if content {6} else {3},"fit":"cover","z":1}));
        }
    }
    json!({"version":1,"timestamp_ns":pts.to_string(),"canvas":{"width":width,"height":height},"surface":if screen.is_some(){None}else{state.surface.as_deref()},"slate":state.slate,"layers":layers,"cameraPage":page,"cameraPages":pages})
}
