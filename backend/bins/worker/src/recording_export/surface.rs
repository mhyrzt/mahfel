use super::{
    notes,
    replay::{Replay, ns},
};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use yrs::{Map, ReadTxn, Transact};

pub(super) fn content(state: &Replay, assets: &Value, width: i32, height: i32) -> Result<Value> {
    if let Some(slate) = state.slate {
        return Ok(
            json!({"type":"slate","state":slate,"text":if slate=="paused" {"Recording paused"} else {"Recording unavailable"}}),
        );
    }
    match state.surface.as_deref() {
        Some("whiteboard") => {
            let mut elements = Vec::new();
            if let Some(doc) = state.document("whiteboard") {
                let txn = doc.transact();
                if let Some(map) = txn.get_map("elements") {
                    for (_, value) in map.iter(&txn) {
                        let serialized = value.to_string(&txn);
                        let serialized: String =
                            serde_json::from_str(&serialized).unwrap_or(serialized);
                        let element: Value = serde_json::from_str(&serialized)?;
                        if element["isDeleted"] == true {
                            continue;
                        }
                        elements.push(element);
                    }
                }
            }
            ensure!(elements.len() <= 2000, "whiteboard element limit");
            elements.sort_by(|a, b| {
                a["index"]
                    .as_str()
                    .cmp(&b["index"].as_str())
                    .then_with(|| a["id"].as_str().cmp(&b["id"].as_str()))
            });
            let viewport = if let Some(viewport) = state.viewports.get("whiteboard") {
                json!({"x":viewport["x"].as_f64().unwrap_or_default(),"y":viewport["y"].as_f64().unwrap_or_default(),"zoom":viewport["zoom"].as_f64().unwrap_or(1.0)})
            } else {
                let left = elements
                    .iter()
                    .filter_map(|item| item["x"].as_f64())
                    .reduce(f64::min)
                    .unwrap_or_default();
                let top = elements
                    .iter()
                    .filter_map(|item| item["y"].as_f64())
                    .reduce(f64::min)
                    .unwrap_or_default();
                let right = elements
                    .iter()
                    .filter_map(|item| Some(item["x"].as_f64()? + item["width"].as_f64()?))
                    .reduce(f64::max)
                    .unwrap_or(left + 1.0);
                let bottom = elements
                    .iter()
                    .filter_map(|item| Some(item["y"].as_f64()? + item["height"].as_f64()?))
                    .reduce(f64::max)
                    .unwrap_or(top + 1.0);
                let zoom = (f64::from(width) / (right - left).max(1.0))
                    .min(f64::from(height) / (bottom - top).max(1.0))
                    .clamp(0.01, 10.0);
                json!({"x":-left+(f64::from(width)/zoom-(right-left))/2.0,"y":-top+(f64::from(height)/zoom-(bottom-top))/2.0,"zoom":zoom})
            };
            let elements = elements.iter().map(|element| {
                let mut rendered = json!({"type":element["type"],"x":element["x"],"y":element["y"],"width":element["width"],"height":element["height"],"angle":element["angle"].as_f64().unwrap_or_default(),"stroke":element["strokeColor"].as_str().unwrap_or("#1e1e1e"),"fill":element["backgroundColor"].as_str().unwrap_or("transparent"),"stroke_width":element["strokeWidth"].as_f64().unwrap_or(1.0),"opacity":element["opacity"].as_f64().unwrap_or(100.0)/100.0,"seed":element["seed"].as_u64().unwrap_or(1).max(1),"roughness":element["roughness"].as_f64().unwrap_or(1.0),"fill_style":element["fillStyle"].as_str().unwrap_or("solid"),"stroke_style":element["strokeStyle"].as_str().unwrap_or("solid")});
                for (source,target) in [("points","points"),("text","text"),("fontSize","font_size"),("textAlign","align"),("startArrowhead","start_arrowhead"),("endArrowhead","end_arrowhead")] { if !element[source].is_null() { rendered[target] = element[source].clone(); } }
                if element["type"] == "image" { rendered["asset"] = assets[element["fileId"].as_str().context("whiteboard image id")?]["render"].clone(); }
                Ok(rendered)
            }).collect::<Result<Vec<_>>>()?;
            Ok(json!({"type":"whiteboard","elements":elements,"viewport":viewport}))
        }
        Some("notes") => {
            let blocks = state
                .document("notes")
                .map(|doc| notes::blocks(doc, assets))
                .transpose()?
                .unwrap_or_default();
            Ok(
                json!({"type":"notes","direction":"ltr","document":{"blocks":blocks},"viewport":{"scroll_y":state.viewports.get("notes").and_then(|view|view["y"].as_f64()).unwrap_or_default().max(0.0)}}),
            )
        }
        Some("presentation") => {
            if let Some((id, asset)) = presentation_asset(state, assets)? {
                let mime = asset["mime"].as_str().context("presentation mime")?;
                if mime.starts_with("video/") {
                    return Ok(json!({"type":"slate","state":"empty","text":""}));
                }
                let page = if state.presentation["file_ids"]
                    .as_array()
                    .context("presentation files")?
                    .len()
                    == 1
                {
                    state.presentation["page"].as_i64().unwrap_or(1)
                } else {
                    1
                };
                ensure!(
                    !asset["render"].is_null(),
                    "presentation asset reference missing: {id}"
                );
                let mut content = json!({"type":if mime=="application/pdf" {"document_page"}else{"image"},"asset":asset["render"],"fit":"contain"});
                if mime == "application/pdf" {
                    content["page"] = json!(page);
                }
                return Ok(content);
            }
            Ok(json!({"type":"slate","state":"empty","text":"Archived presentation unavailable"}))
        }
        _ => Ok(json!({"type":"slate","state":"empty","text":""})),
    }
}

fn presentation_asset<'a>(
    state: &Replay,
    assets: &'a Value,
) -> Result<Option<(String, &'a Value)>> {
    if state.presentation.is_null() {
        return Ok(None);
    }
    let files = state.presentation["file_ids"]
        .as_array()
        .context("presentation files")?;
    let index = if files.len() == 1 {
        0
    } else {
        state.presentation["page"]
            .as_u64()
            .unwrap_or(1)
            .saturating_sub(1) as usize
    };
    Ok(files
        .get(index)
        .and_then(Value::as_str)
        .and_then(|id| assets.get(id).map(|asset| (id.into(), asset))))
}

pub(super) fn presentation_video(state: &Replay, assets: &Value, pts: u64) -> Result<Value> {
    if let Some((id, asset)) = presentation_asset(state, assets)?
        && asset["mime"]
            .as_str()
            .is_some_and(|mime| mime.starts_with("video/"))
    {
        let position = state.presentation["position_ms"]
            .as_f64()
            .unwrap_or_default()
            + if state.presentation["paused"] == true {
                0.0
            } else {
                (ns(&state.input["start_ns"])? + i128::from(pts)
                    - ns(&state.presentation["anchor_ns"])?) as f64
                    / 1_000_000.0
                    * state.presentation["rate"].as_f64().unwrap_or(1.0)
            };
        return Ok(
            json!({"file_id":id,"position_ms":position.max(0.0),"paused":state.presentation["paused"],"rate":state.presentation["rate"]}),
        );
    }
    Ok(Value::Null)
}

pub(super) fn annotations(
    state: &Replay,
    scene: &Value,
    tracks: &[Value],
    width: i32,
    height: i32,
) -> Vec<Value> {
    let mut values = Vec::new();
    if let Some(layers) = scene["layers"].as_array() {
        for layer in layers {
            if let Some(track) = tracks
                .iter()
                .find(|track| track["id"] == layer["source"] && track["source"] == "camera")
            {
                let name = state.input["actors"][track["actorId"].as_str().unwrap_or_default()]
                    .as_str()
                    .unwrap_or("Participant");
                values.push(json!({"text":name.chars().take(256).collect::<String>(),"x":layer["x"].as_i64().unwrap_or_default()+8,"y":(layer["y"].as_i64().unwrap_or_default()+layer["height"].as_i64().unwrap_or_default()-40).max(0),"width":(layer["width"].as_i64().unwrap_or(100)-16).max(16),"font_size":18,"foreground":"#f4f3f8","background":"#101018dc"}));
            }
        }
    }
    if state.slate.is_none() {
        for (index, toast) in state.toasts.iter().enumerate() {
            values.push(json!({"text":format!("{}\n{}",toast["author"].as_str().unwrap_or("Participant"),toast["body"].as_str().unwrap_or_default()).chars().take(1024).collect::<String>(),"x":24,"y":(height-170*(state.toasts.len()-index) as i32).max(0),"width":460.min(width-48).max(16),"font_size":20,"foreground":"#f4f3f8","background":"#181824ef"}));
        }
    }
    if state.input["partial"] == true {
        values.push(json!({"text":"Partial recording — some content may be unavailable","x":(width-480).max(0),"y":12,"width":480.min(width),"font_size":14,"foreground":"#ffffff","background":"#272734"}));
    }
    values
}
