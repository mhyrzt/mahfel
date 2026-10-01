use super::Replay;
use crate::recording_export::{scene, surface};
use serde_json::{Value, json};

fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/replay.json")).expect("recorded Yjs fixture")
}

fn assets() -> Value {
    json!({
        "image": {"mime":"image/png","render":{"url":"http://storage.test/image","sha256":"0".repeat(64)}},
        "pdf": {"mime":"application/pdf","render":{"url":"http://storage.test/pdf","sha256":"1".repeat(64)}},
        "video": {"mime":"video/webm"}
    })
}

#[test]
fn replays_yjs_presentations_pause_resume_and_backward_seek() -> anyhow::Result<()> {
    let mut replay = Replay::new(fixture())?;
    replay.advance(0)?;
    let board = surface::content(&replay, &assets(), 1920, 900)?;
    assert_eq!(board["type"], "whiteboard");
    assert_eq!(board["elements"][0]["type"], "rectangle");
    assert_eq!(board["elements"][0]["x"], 100);
    replay.advance(1_500_000_000)?;
    assert_eq!(
        surface::content(&replay, &assets(), 1920, 900)?["type"],
        "image"
    );
    replay.advance(2_500_000_000)?;
    assert_eq!(
        surface::content(&replay, &assets(), 1920, 900)?["type"],
        "document_page"
    );
    replay.advance(4_250_000_000)?;
    assert_eq!(
        surface::presentation_video(&replay, &assets(), 4_250_000_000)?["position_ms"],
        250.0
    );
    replay.advance(5_500_000_000)?;
    assert_eq!(replay.surface.as_deref(), Some("whiteboard"));
    replay.advance(6_500_000_000)?;
    assert_eq!(replay.slate, Some("paused"));
    assert!(replay.toasts.is_empty());
    replay.advance(7_000_000_000)?;
    let notes = surface::content(&replay, &assets(), 1920, 900)?;
    assert_eq!(notes["type"], "notes");
    assert!(notes["document"]["blocks"].to_string().contains("یادداشت"));
    let blocks = notes["document"]["blocks"]
        .as_array()
        .expect("notes blocks");
    assert!(
        blocks
            .iter()
            .any(|block| block["direction"] == "rtl" && block["runs"][0]["bold"] == true)
    );
    let markers = blocks
        .iter()
        .filter(|block| block["type"] == "list_item")
        .map(|block| block["marker"].clone())
        .collect::<Vec<_>>();
    assert_eq!(markers, vec![json!("3."), json!("4.")]);
    replay.advance(0)?;
    assert_eq!(replay.surface.as_deref(), Some("whiteboard"));
    assert!(replay.presentation.is_null());
    assert!(replay.toasts.is_empty());
    Ok(())
}

#[test]
fn chat_edits_deletes_and_expiry_follow_the_recorded_clock() -> anyhow::Result<()> {
    let mut input = fixture();
    input["events"] = json!([
        {"generation":1,"event_kind":"chat","source_id":"a","at_ns":"0","payload":{"kind":"created","message_id":"first","author_actor_id":"sara","body":"original"}},
        {"generation":1,"event_kind":"chat","source_id":"b","at_ns":"1000000000","payload":{"kind":"edited","message_id":"first","body":"edited"}},
        {"generation":1,"event_kind":"chat","source_id":"c","at_ns":"2000000000","payload":{"kind":"deleted","message_id":"first"}}
    ]);
    let mut replay = Replay::new(input)?;
    replay.advance(0)?;
    assert_eq!(replay.toasts[0]["author"], "سارا");
    replay.advance(1_000_000_000)?;
    assert_eq!(replay.toasts[0]["body"], "edited");
    replay.advance(2_000_000_000)?;
    assert!(replay.toasts.is_empty());
    replay.advance(0)?;
    assert_eq!(replay.toasts[0]["body"], "original");
    Ok(())
}

#[test]
fn outage_gaps_are_distinct_from_explicit_pause() -> anyhow::Result<()> {
    let mut input = fixture();
    input["intervals"][0]["loss_reason"] = json!("media_node_lost");
    let mut replay = Replay::new(input)?;
    replay.advance(6_500_000_000)?;
    assert_eq!(replay.slate, Some("outage"));
    assert_eq!(
        surface::content(&replay, &assets(), 1920, 1080)?["state"],
        "outage"
    );
    Ok(())
}

#[test]
fn camera_pages_rotate_and_latest_screen_takes_priority() -> anyhow::Result<()> {
    let mut input = fixture();
    input["snapshots"] = json!([]);
    input["events"] = json!([]);
    let mut replay = Replay::new(input)?;
    replay.advance(0)?;
    let mut tracks = (0..10).map(|id| json!({"id":format!("camera-{id:02}"),"actorId":format!("actor-{id:02}"),"source":"camera"})).collect::<Vec<_>>();
    let first = scene::describe(&replay, 0, &tracks, 1920, 1080);
    assert_eq!(first["layers"].as_array().map(Vec::len), Some(9));
    let second = scene::describe(&replay, 10_000_000_000, &tracks, 1920, 1080);
    assert_eq!(second["layers"][0]["source"], "camera-09");
    tracks.extend([
        json!({"id":"old-screen","actorId":"a","source":"screen","started_ns":"9000000000"}),
        json!({"id":"new-screen","actorId":"z","source":"screen","started_ns":"10000000000"}),
    ]);
    let scene = scene::describe(&replay, 0, &tracks, 1920, 1080);
    assert_eq!(scene["layers"][0]["source"], "new-screen");
    assert_eq!(scene["layers"].as_array().map(Vec::len), Some(7));
    Ok(())
}
