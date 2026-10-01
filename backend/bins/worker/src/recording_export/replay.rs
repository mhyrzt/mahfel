use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use yrs::{Doc, Map, ReadTxn, Text, Transact, Update, updates::decoder::Decode};

#[cfg(test)]
#[path = "replay_tests.rs"]
mod tests;

pub(super) fn ns(value: &Value) -> Result<i128> {
    value
        .as_str()
        .context("timestamp string")?
        .parse()
        .context("timestamp integer")
}

pub(super) struct Document {
    pub kind: String,
    pub doc: Doc,
}

pub(super) struct Replay {
    pub input: Value,
    pub documents: BTreeMap<String, Document>,
    pub viewports: BTreeMap<String, Value>,
    pub presentation: Value,
    pub toasts: Vec<Value>,
    pub surface: Option<String>,
    pub slate: Option<&'static str>,
    workspace: Option<String>,
    generation: Option<i64>,
    events: Vec<Value>,
    cursor: usize,
    last: i128,
}

impl Replay {
    pub fn new(input: Value) -> Result<Self> {
        ensure!(input["version"] == 1, "unsupported replay version");
        let mut events = input["events"].as_array().context("replay events")?.clone();
        ensure!(events.len() <= 100000, "event resource limit");
        for event in &events {
            ns(&event["at_ns"])?;
        }
        events.sort_by(|a, b| {
            ns(&a["at_ns"])
                .unwrap_or_default()
                .cmp(&ns(&b["at_ns"]).unwrap_or_default())
                .then_with(|| sequence(&a["payload"]).cmp(&sequence(&b["payload"])))
                .then_with(|| a["source_id"].as_str().cmp(&b["source_id"].as_str()))
        });
        Ok(Self {
            input,
            documents: BTreeMap::new(),
            viewports: BTreeMap::new(),
            presentation: Value::Null,
            toasts: Vec::new(),
            surface: None,
            slate: None,
            workspace: None,
            generation: None,
            events,
            cursor: 0,
            last: -1,
        })
    }

    fn clear(&mut self) {
        self.documents.clear();
        self.viewports.clear();
        self.presentation = Value::Null;
        self.toasts.clear();
        self.surface = None;
        self.workspace = None;
    }

    pub fn document(&self, kind: &str) -> Option<&Doc> {
        self.documents
            .values()
            .find(|item| item.kind == kind)
            .map(|item| &item.doc)
    }

    pub fn advance(&mut self, output: i128) -> Result<()> {
        ensure!(
            output >= 0 && output <= ns(&self.input["duration_ns"])?,
            "replay bounds"
        );
        let at = ns(&self.input["start_ns"])? + output;
        if at < self.last {
            self.clear();
            self.cursor = 0;
            self.generation = None;
        }
        self.last = at;
        let intervals = self.input["intervals"]
            .as_array()
            .context("replay intervals")?;
        let interval = intervals
            .iter()
            .find(|item| {
                ns(&item["start_ns"]).is_ok_and(|start| start <= at)
                    && ns(&item["end_ns"]).is_ok_and(|end| at < end)
            })
            .cloned();
        let Some(interval) = interval else {
            self.slate = Some(
                if intervals
                    .iter()
                    .rev()
                    .find(|item| ns(&item["end_ns"]).is_ok_and(|end| end <= at))
                    .is_some_and(|item| item["loss_reason"].is_null())
                {
                    "paused"
                } else {
                    "outage"
                },
            );
            self.toasts.clear();
            return Ok(());
        };
        self.slate = None;
        let generation = interval["generation"].as_i64().context("generation")?;
        if self.generation != Some(generation) {
            self.clear();
            self.generation = Some(generation);
            let snapshots = self.input["snapshots"]
                .as_array()
                .context("snapshots")?
                .iter()
                .filter(|item| item["generation"] == generation)
                .cloned()
                .collect::<Vec<_>>();
            for snapshot in snapshots
                .iter()
                .filter(|item| item["snapshot_kind"] == "document")
            {
                let kind = snapshot["payload"]["kind"]
                    .as_str()
                    .context("document kind")?;
                ensure!(
                    snapshot["payload"]["schema_version"].is_null()
                        || snapshot["payload"]["schema_version"] == 1,
                    "document schema"
                );
                ensure!(matches!(kind, "notes" | "whiteboard"), "document kind");
                let doc = Doc::new();
                if kind == "notes" {
                    doc.get_or_insert_text("root");
                } else {
                    doc.get_or_insert_map("elements");
                    doc.get_or_insert_map("image-assets");
                }
                self.documents.insert(
                    snapshot["source_id"]
                        .as_str()
                        .context("document id")?
                        .into(),
                    Document {
                        kind: kind.into(),
                        doc,
                    },
                );
            }
            for snapshot in snapshots
                .iter()
                .filter(|item| item["snapshot_kind"] == "checkpoint")
            {
                self.update(&snapshot["payload"])?;
            }
            let mut updates = snapshots
                .iter()
                .filter(|item| item["snapshot_kind"] == "document_update")
                .collect::<Vec<_>>();
            updates.sort_by_key(|item| sequence(&item["payload"]));
            for item in updates {
                self.update(&item["payload"])?;
            }
            let mut viewports = snapshots
                .iter()
                .filter(|item| item["snapshot_kind"] == "viewport")
                .collect::<Vec<_>>();
            viewports.sort_by_key(|item| sequence(&item["payload"]));
            for item in viewports {
                self.viewport(&item["payload"]["payload"]);
            }
            if let Some(item) = snapshots.iter().find(|item| {
                item["snapshot_kind"] == "presentation"
                    && !item["payload"]["selection_event_id"].is_null()
            }) {
                let mut assets = snapshots
                    .iter()
                    .filter(|item| item["snapshot_kind"] == "presentation_asset")
                    .collect::<Vec<_>>();
                assets.sort_by_key(|item| item["payload"]["ordinal"].as_i64().unwrap_or_default());
                let p = &item["payload"];
                self.presentation = json!({ "file_ids": assets.iter().map(|item| item["payload"]["file_id"].clone()).collect::<Vec<_>>(), "page": p["current_page"].as_i64().unwrap_or(1), "position_ms": p["playback_position_ms"].as_f64().unwrap_or_default(), "paused": p["playback_paused"] != false, "rate": p["playback_rate"].as_f64().unwrap_or(1.0), "anchor_ns": p["playback_anchor_ns"].as_str().unwrap_or(interval["start_ns"].as_str().context("interval start")?) });
                self.surface = Some("presentation".into());
            }
            if self.surface.is_none() {
                if self.document("whiteboard").is_some_and(|doc| {
                    doc.transact()
                        .get_map("elements")
                        .is_some_and(|map| map.len(&doc.transact()) > 0)
                }) {
                    self.surface = Some("whiteboard".into());
                } else if self
                    .document("notes")
                    .is_some_and(|doc| doc.get_or_insert_text("root").len(&doc.transact()) > 0)
                {
                    self.surface = Some("notes".into());
                }
            }
        }
        if self.workspace.is_none()
            && self
                .surface
                .as_deref()
                .is_some_and(|kind| matches!(kind, "notes" | "whiteboard"))
        {
            self.workspace = self.surface.clone();
        }
        while self.cursor < self.events.len() && ns(&self.events[self.cursor]["at_ns"])? <= at {
            let event = self.events[self.cursor].clone();
            self.cursor += 1;
            if event["generation"] != generation {
                continue;
            }
            let payload = &event["payload"];
            if event["event_kind"] == "collaboration" {
                match payload["kind"].as_str() {
                    Some("document_update") => {
                        self.update(payload)?;
                        if let Some(doc) = self
                            .documents
                            .get(payload["document_id"].as_str().context("document id")?)
                        {
                            self.surface = Some(doc.kind.clone());
                            self.workspace = self.surface.clone();
                        }
                    }
                    Some("viewport") => self.viewport(&payload["payload"]),
                    Some("presentation") => {
                        self.present(&payload["payload"], ns(&event["at_ns"])?)?
                    }
                    _ => {}
                }
            } else if event["event_kind"] == "chat" {
                self.chat(payload, ns(&event["at_ns"])?)?;
            }
        }
        self.toasts
            .retain(|item| ns(&item["expires_ns"]).is_ok_and(|expiry| expiry > at));
        Ok(())
    }

    fn update(&self, value: &Value) -> Result<()> {
        let document = self
            .documents
            .get(value["document_id"].as_str().context("document id")?)
            .context("recorded document missing")?;
        let encoded = value["update_bytes"]
            .as_str()
            .context("document bytes")?
            .strip_prefix("\\x")
            .context("document byte encoding")?;
        ensure!(
            encoded.len() % 2 == 0 && encoded.len() <= 16 * 1024 * 1024,
            "document resource limit"
        );
        let bytes = encoded
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| {
                u8::from_str_radix(std::str::from_utf8(pair)?, 16).map_err(anyhow::Error::from)
            })
            .collect::<Result<Vec<_>>>()?;
        document
            .doc
            .transact_mut()
            .apply_update(Update::decode_v1(&bytes)?)?;
        Ok(())
    }
    fn viewport(&mut self, value: &Value) {
        if let Some(kind @ ("notes" | "whiteboard")) = value["surface"].as_str() {
            self.viewports.insert(kind.into(), value.clone());
            self.surface = Some(kind.into());
            self.workspace = self.surface.clone();
        }
    }
    fn present(&mut self, value: &Value, at: i128) -> Result<()> {
        match value["action"].as_str() {
            Some("stop") => {
                self.presentation = Value::Null;
                self.surface = self.workspace.clone();
                return Ok(());
            }
            Some("select") => {
                ensure!(value["file_ids"].is_array(), "presentation file ids");
                self.presentation = json!({"file_ids": value["file_ids"], "page": 1, "position_ms": 0, "paused": true, "rate": 1, "anchor_ns": at.to_string()});
            }
            Some("page") if !self.presentation.is_null() => {
                self.presentation["page"] = value["page"].clone()
            }
            Some("playback") if !self.presentation.is_null() => {
                for key in ["position_ms", "paused", "rate"] {
                    self.presentation[key] = value[key].clone();
                }
                self.presentation["anchor_ns"] = json!(at.to_string());
            }
            _ => {}
        }
        if !self.presentation.is_null() {
            self.surface = Some("presentation".into());
        }
        Ok(())
    }
    fn chat(&mut self, value: &Value, at: i128) -> Result<()> {
        let id = value["message_id"].as_str().context("chat id")?;
        let existing = self.toasts.iter().position(|item| item["id"] == id);
        match value["kind"].as_str() {
            Some("deleted") => {
                if let Some(index) = existing {
                    self.toasts.remove(index);
                }
            }
            Some("edited") => {
                if let Some(index) = existing {
                    self.toasts[index]["body"] = value["body"].clone();
                }
            }
            Some("created") => {
                self.toasts.push(json!({"id":id, "author": self.input["actors"][value["author_actor_id"].as_str().context("chat actor")?].as_str().unwrap_or("Participant"),"body":value["body"],"expires_ns":(at+6_000_000_000).to_string()}));
                if self.toasts.len() > 3 {
                    self.toasts.remove(0);
                }
            }
            _ => {}
        }
        Ok(())
    }
}

fn sequence(value: &Value) -> i64 {
    value["event_sequence"]
        .as_i64()
        .or_else(|| value["event_seq"].as_i64())
        .unwrap_or_default()
}
