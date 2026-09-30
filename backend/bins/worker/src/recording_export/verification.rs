use anyhow::{Context, Result, ensure};
use gstreamer::{self as gst, prelude::*};
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

pub(super) fn verify(path: &Path, duration_ns: u64) -> Result<()> {
    let pipeline = gst::parse::launch(
        "uridecodebin name=input input. ! queue ! videoconvert ! video/x-raw ! fakesink name=video sync=false input. ! queue ! audioconvert ! audio/x-raw ! fakesink name=audio sync=false",
    )?
    .downcast::<gst::Pipeline>()
    .map_err(|_| anyhow::anyhow!("invalid verification pipeline"))?;
    pipeline
        .by_name("input")
        .context("verification input")?
        .set_property("uri", gst::glib::filename_to_uri(path, None)?);
    let mut streams = Vec::new();
    for name in ["video", "audio"] {
        let first = Arc::new(AtomicU64::new(u64::MAX));
        let last = Arc::new(AtomicU64::new(0));
        let sink = pipeline.by_name(name).context("verification stream")?;
        sink.set_property("signal-handoffs", true);
        let starts = first.clone();
        let ends = last.clone();
        sink.connect("handoff", false, move |values| {
            if let Some(buffer) = values
                .get(1)
                .and_then(|value| value.get::<gst::Buffer>().ok())
                && let Some(pts) = buffer.pts()
            {
                let at = pts.nseconds();
                starts.fetch_min(at, Ordering::Relaxed);
                ends.fetch_max(
                    at.saturating_add(
                        buffer
                            .duration()
                            .map(|duration| duration.nseconds())
                            .unwrap_or(0),
                    ),
                    Ordering::Relaxed,
                );
            }
            None
        });
        streams.push((first, last));
    }
    let result = (|| {
        pipeline.set_state(gst::State::Playing)?;
        let message = pipeline
            .bus()
            .context("verification bus")?
            .timed_pop_filtered(
                gst::ClockTime::from_seconds(
                    duration_ns.div_ceil(1_000_000_000).saturating_add(60),
                ),
                &[gst::MessageType::Eos, gst::MessageType::Error],
            )
            .context("MP4 verification timeout")?;
        if let gst::MessageView::Error(error) = message.view() {
            anyhow::bail!("MP4 verification failed: {}", error.error());
        }
        let actual = pipeline
            .query_duration::<gst::ClockTime>()
            .context("MP4 duration missing")?
            .nseconds();
        ensure!(
            actual.abs_diff(duration_ns) <= 100_000_000,
            "MP4 duration mismatch"
        );
        for (first, last) in &streams {
            ensure!(
                first.load(Ordering::Relaxed) != u64::MAX && last.load(Ordering::Relaxed) > 0,
                "MP4 stream missing"
            );
        }
        ensure!(
            streams[0]
                .0
                .load(Ordering::Relaxed)
                .abs_diff(streams[1].0.load(Ordering::Relaxed))
                <= 100_000_000,
            "MP4 stream start mismatch"
        );
        ensure!(
            streams[0]
                .1
                .load(Ordering::Relaxed)
                .abs_diff(streams[1].1.load(Ordering::Relaxed))
                <= 100_000_000,
            "MP4 stream end mismatch"
        );
        Ok(())
    })();
    let _ = pipeline.set_state(gst::State::Null);
    result
}
