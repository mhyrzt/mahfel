use crate::RtcError;
use gst::prelude::*;
use gstreamer as gst;
use serde_json::Value;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone)]
pub struct CaptureClock {
    pub unix_origin_ns: u64,
    clock: gst::Clock,
    base: gst::ClockTime,
}

impl CaptureClock {
    pub fn new() -> Result<Self, RtcError> {
        gst::init().map_err(failure)?;
        let clock = gst::SystemClock::obtain();
        let base = clock.time();
        let unix_origin_ns = u64::try_from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(failure)?
                .as_nanos(),
        )
        .map_err(failure)?;
        Ok(Self {
            unix_origin_ns,
            clock,
            base,
        })
    }

    pub fn configure(&self, pipeline: &gst::Pipeline) {
        pipeline.use_clock(Some(&self.clock));
        pipeline.set_start_time(gst::ClockTime::NONE);
        pipeline.set_base_time(self.base);
    }
}

#[derive(Debug, Clone)]
pub struct CaptureFragment {
    pub path: PathBuf,
    pub start_ns: u64,
    pub end_ns: u64,
}

pub struct CapturePipeline {
    pipeline: gst::Pipeline,
    pending: Arc<Mutex<HashMap<PathBuf, u64>>>,
    timestamps: Arc<Mutex<Option<(u64, u64)>>>,
    ended: std::sync::atomic::AtomicBool,
    pub codec: String,
    pub container: String,
}

fn failure(error: impl std::fmt::Display) -> RtcError {
    RtcError::Recording(error.to_string())
}

impl CapturePipeline {
    pub fn start(
        parameters: &Value,
        directory: &Path,
        clock: &CaptureClock,
    ) -> Result<Self, RtcError> {
        std::fs::create_dir_all(directory).map_err(failure)?;
        let codecs = parameters["codecs"]
            .as_array()
            .ok_or_else(|| failure("missing codecs"))?;
        let codec = codecs
            .iter()
            .find(|codec| {
                codec["mimeType"]
                    .as_str()
                    .is_some_and(|mime| !mime.ends_with("/rtx"))
            })
            .ok_or_else(|| failure("missing primary codec"))?;
        let mime = codec["mimeType"]
            .as_str()
            .ok_or_else(|| failure("missing mime"))?;
        let (encoding, depay, parser, mux, extension, media) =
            match mime.to_ascii_lowercase().as_str() {
                "audio/opus" => (
                    "OPUS",
                    "rtpopusdepay",
                    "opusparse",
                    "webmmux",
                    "webm",
                    "audio",
                ),
                "video/vp8" => ("VP8", "rtpvp8depay", "identity", "webmmux", "webm", "video"),
                "video/h264" => (
                    "H264",
                    "rtph264depay",
                    "h264parse",
                    "matroskamux",
                    "mkv",
                    "video",
                ),
                _ => return Err(failure("unsupported capture codec")),
            };
        let payload = codec["payloadType"]
            .as_u64()
            .filter(|value| *value <= 127)
            .ok_or_else(|| failure("invalid payload type"))?;
        let rate = codec["clockRate"]
            .as_u64()
            .filter(|value| *value > 0 && *value <= u64::from(u32::MAX))
            .ok_or_else(|| failure("invalid clock rate"))?;
        let channels = if media == "audio" {
            ",encoding-params=(string)2"
        } else {
            ""
        };
        let sink_pad = if media == "audio" {
            "fragments.audio_0"
        } else {
            "fragments.video"
        };
        let description = format!(
            "rtpbin name=rtp latency=50 ntp-sync=true add-reference-timestamp-meta=true \
             udpsrc name=rtp_source address=127.0.0.1 port=0 buffer-size=262144 \
               caps=\"application/x-rtp,media=(string){media},encoding-name=(string){encoding},clock-rate=(int){rate},payload=(int){payload}{channels}\" ! rtp.recv_rtp_sink_0 \
             udpsrc name=rtcp_source address=127.0.0.1 port=0 buffer-size=65536 caps=application/x-rtcp ! rtp.recv_rtcp_sink_0 \
             rtp.send_rtcp_src_0 ! udpsink name=rtcp_feedback host=127.0.0.1 port=9 sync=false async=false \
             rtp. ! {depay} ! {parser} ! identity name=encoded ! queue max-size-bytes=4194304 max-size-buffers=0 max-size-time=2000000000 ! {sink_pad} \
             splitmuxsink name=fragments muxer-factory={mux} async-finalize=true max-size-time=5000000000 max-size-bytes=16777216"
        );
        let pipeline = gst::parse::launch(&description)
            .map_err(failure)?
            .downcast::<gst::Pipeline>()
            .map_err(|_| failure("invalid capture pipeline"))?;
        let fragments = pipeline
            .by_name("fragments")
            .ok_or_else(|| failure("missing fragment sink"))?;
        let location = directory.join(format!("%06d.{extension}"));
        fragments.set_property(
            "location",
            location
                .to_str()
                .ok_or_else(|| failure("invalid spool path"))?,
        );
        let timestamps = Arc::new(Mutex::new(None));
        let observed = Arc::clone(&timestamps);
        pipeline
            .by_name("encoded")
            .and_then(|element| element.static_pad("src"))
            .ok_or_else(|| failure("missing encoded media pad"))?
            .add_probe(gst::PadProbeType::BUFFER, move |pad, probe| {
                if let Some(pts) = probe
                    .buffer()
                    .and_then(|buffer| buffer.pts())
                    .and_then(|pts| {
                        pad.sticky_event::<gst::event::Segment>(0)
                            .and_then(|event| {
                                event
                                    .segment()
                                    .downcast_ref::<gst::ClockTime>()
                                    .and_then(|segment| segment.to_running_time(pts))
                            })
                    })
                    && let Ok(mut bounds) = observed.lock()
                {
                    let first = bounds.map_or(pts.nseconds(), |(first, _)| first);
                    *bounds = Some((first, pts.nseconds()));
                }
                gst::PadProbeReturn::Ok
            });
        clock.configure(&pipeline);
        let capture = Self {
            pipeline,
            pending: Arc::new(Mutex::new(HashMap::new())),
            timestamps,
            ended: std::sync::atomic::AtomicBool::new(false),
            codec: mime.to_owned(),
            container: extension.to_owned(),
        };
        capture
            .pipeline
            .set_state(gst::State::Playing)
            .map_err(failure)?;
        Ok(capture)
    }

    fn port(&self, name: &str) -> Result<u16, RtcError> {
        let element = self
            .pipeline
            .by_name(name)
            .ok_or_else(|| failure("missing UDP source"))?;
        u16::try_from(element.property::<i32>("port")).map_err(failure)
    }
    pub fn rtp_port(&self) -> Result<u16, RtcError> {
        self.port("rtp_source")
    }
    pub fn rtcp_port(&self) -> Result<u16, RtcError> {
        self.port("rtcp_source")
    }

    pub fn set_rtcp_feedback_port(&self, port: u16) -> Result<(), RtcError> {
        self.pipeline
            .by_name("rtcp_feedback")
            .ok_or_else(|| failure("missing RTCP feedback sink"))?
            .set_property("port", i32::from(port));
        Ok(())
    }

    pub fn media_bounds(&self) -> Result<Option<(u64, u64)>, RtcError> {
        self.timestamps
            .lock()
            .map(|bounds| *bounds)
            .map_err(failure)
    }

    pub fn drain(&self) -> Result<Vec<CaptureFragment>, RtcError> {
        let bus = self
            .pipeline
            .bus()
            .ok_or_else(|| failure("missing capture bus"))?;
        let mut result = Vec::new();
        for message in bus.iter() {
            match message.view() {
                gst::MessageView::Eos(..) => {
                    self.ended.store(true, std::sync::atomic::Ordering::Release);
                }
                gst::MessageView::Error(error) => return Err(failure(error.error())),
                gst::MessageView::Element(element) => {
                    let Some(data) = element.structure() else {
                        continue;
                    };
                    if !matches!(
                        data.name().as_str(),
                        "splitmuxsink-fragment-opened" | "splitmuxsink-fragment-closed"
                    ) {
                        continue;
                    }
                    let location = PathBuf::from(data.get::<String>("location").map_err(failure)?);
                    let time = data.get::<u64>("running-time").map_err(failure)?;
                    let mut pending = self.pending.lock().map_err(failure)?;
                    if data.name() == "splitmuxsink-fragment-opened" {
                        pending.insert(location, time);
                    } else if let Some(start_ns) = pending.remove(&location) {
                        result.push(CaptureFragment {
                            path: location,
                            start_ns,
                            end_ns: time.max(start_ns),
                        });
                    }
                }
                _ => {}
            }
        }
        Ok(result)
    }

    pub fn ended(&self) -> bool {
        self.ended.load(std::sync::atomic::Ordering::Acquire)
    }

    pub fn seal(&self) -> Result<(), RtcError> {
        if !self.pipeline.send_event(gst::event::Eos::new()) {
            return Err(failure("capture refused EOS"));
        }
        Ok(())
    }
}

impl Drop for CapturePipeline {
    fn drop(&mut self) {
        let _ = self.pipeline.set_state(gst::State::Null);
    }
}
