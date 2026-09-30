use anyhow::{Context, Result, ensure};
use gstreamer::{self as gst, prelude::*};
use std::path::Path;

pub(super) struct VideoFrame {
    pub width: i32,
    pub height: i32,
    pub bytes: Vec<u8>,
}
struct Frame {
    pts: u64,
    sample: gst::Sample,
}
pub(super) struct Decoder {
    pipeline: gst::Pipeline,
    sink: gst::Element,
    first_pts: Option<u64>,
    previous: Option<Frame>,
    next: Option<Frame>,
    audio: bool,
    ended: bool,
    allow_missing_audio: bool,
}
impl Decoder {
    pub(super) fn open(path: &Path, audio: bool) -> Result<Self> {
        let format = if audio {
            "audioconvert ! audioresample ! audio/x-raw,format=F32LE,rate=48000,channels=2,layout=interleaved"
        } else {
            "videoconvert ! video/x-raw,format=RGBA"
        };
        let pipeline = gst::parse::launch(&format!("uridecodebin name=input ! queue max-size-buffers=2 max-size-bytes=0 max-size-time=0 ! {format} ! appsink name=output sync=false max-buffers=2"))?.downcast::<gst::Pipeline>().map_err(|_| anyhow::anyhow!("invalid decoder"))?;
        let input = pipeline.by_name("input").context("decoder input")?;
        input.set_property("uri", gst::glib::filename_to_uri(path, None)?);
        let sink = pipeline.by_name("output").context("decoder sink")?;
        pipeline.set_state(gst::State::Playing)?;
        Ok(Self {
            pipeline,
            sink,
            first_pts: None,
            previous: None,
            next: None,
            audio,
            ended: false,
            allow_missing_audio: false,
        })
    }
    pub(super) fn optional_audio(path: &Path) -> Result<Self> {
        let mut decoder = Self::open(path, true)?;
        decoder.allow_missing_audio = true;
        Ok(decoder)
    }
    fn pull(&mut self) -> Result<Option<Frame>> {
        if self.ended {
            return Ok(None);
        }
        let sample = self.sink.emit_by_name::<Option<gst::Sample>>(
            "try-pull-sample",
            &[&gst::ClockTime::from_seconds(5)],
        );
        let Some(sample) = sample else {
            if self.allow_missing_audio
                && self.pipeline.by_name("input").is_some_and(|input| {
                    let pads = input.src_pads();
                    !pads.is_empty()
                        && pads.iter().all(|pad| {
                            pad.current_caps().is_some_and(|caps| {
                                caps.structure(0).is_some_and(|structure| {
                                    !structure.name().starts_with("audio/")
                                })
                            })
                        })
                })
            {
                self.ended = true;
                return Ok(None);
            }
            if let Some(message) = self
                .pipeline
                .bus()
                .context("decoder bus")?
                .pop_filtered(&[gst::MessageType::Error])
                && let gst::MessageView::Error(error) = message.view()
            {
                anyhow::bail!("decode failed: {}", error.error());
            }
            ensure!(self.sink.property::<bool>("eos"), "decoder stalled");
            self.ended = true;
            return Ok(None);
        };
        let pts = sample
            .buffer()
            .and_then(|buffer| buffer.pts())
            .context("decoded timestamp")?
            .nseconds();
        let origin = *self.first_pts.get_or_insert(pts);
        Ok(Some(Frame {
            pts: pts.saturating_sub(origin),
            sample,
        }))
    }
    pub(super) fn seek(&mut self, at_ns: u64) -> Result<()> {
        self.pipeline.seek_simple(
            gst::SeekFlags::FLUSH | gst::SeekFlags::ACCURATE,
            gst::ClockTime::from_nseconds(at_ns),
        )?;
        self.previous = None;
        self.next = None;
        self.ended = false;
        self.first_pts = Some(0);
        Ok(())
    }
    pub(super) fn video(&mut self, at_ns: u64) -> Result<Option<VideoFrame>> {
        ensure!(!self.audio, "not a video decoder");
        if self.next.is_none() {
            self.next = self.pull()?;
        }
        while self.next.as_ref().is_some_and(|frame| frame.pts <= at_ns) {
            self.previous = self.next.take();
            self.next = self.pull()?;
        }
        let Some(frame) = &self.previous else {
            return Ok(None);
        };
        let structure = frame
            .sample
            .caps()
            .and_then(|caps| caps.structure(0))
            .context("video caps")?;
        let width = structure.get::<i32>("width")?;
        let height = structure.get::<i32>("height")?;
        ensure!(
            width > 0 && height > 0 && width <= 8192 && height <= 8192,
            "video dimensions too large"
        );
        let map = frame
            .sample
            .buffer()
            .context("video buffer")?
            .map_readable()?;
        let row = usize::try_from(width)? * 4;
        let rows = usize::try_from(height)?;
        ensure!(map.len() >= row * rows, "short video frame");
        let stride = map.len() / rows;
        let mut bytes = Vec::with_capacity(row * rows);
        for index in 0..rows {
            bytes.extend_from_slice(&map[index * stride..index * stride + row]);
        }
        Ok(Some(VideoFrame {
            width,
            height,
            bytes,
        }))
    }
    pub(super) fn audio(&mut self, at_ns: u64, frames: usize, rate: f64) -> Result<Vec<f32>> {
        ensure!(self.audio, "not an audio decoder");
        let mut output = vec![0.0; frames * 2];
        if self.next.is_none() {
            self.next = self.pull()?;
        }
        for index in 0..frames {
            let target =
                at_ns.saturating_add((index as f64 * rate * 1_000_000_000.0 / 48000.0) as u64);
            while let Some(frame) = &self.next {
                let buffer = frame.sample.buffer().context("audio buffer")?;
                let map = buffer.map_readable()?;
                let length = map.len() / 8;
                let position = target.saturating_sub(frame.pts) * 48000 / 1_000_000_000;
                if target >= frame.pts && position < length as u64 {
                    let offset = usize::try_from(position)? * 8;
                    for channel in 0..2 {
                        let start = offset + channel * 4;
                        output[index * 2 + channel] =
                            f32::from_le_bytes(map[start..start + 4].try_into()?);
                    }
                    break;
                }
                if target < frame.pts {
                    break;
                }
                drop(map);
                self.next = self.pull()?;
            }
        }
        Ok(output)
    }
}
impl Drop for Decoder {
    fn drop(&mut self) {
        let _ = self.pipeline.set_state(gst::State::Null);
    }
}

pub(super) fn crop_camera(
    frame: VideoFrame,
    target_width: i32,
    target_height: i32,
) -> Result<VideoFrame> {
    let mut width = frame.width;
    let mut height = frame.height;
    if i64::from(width) * i64::from(target_height) > i64::from(height) * i64::from(target_width) {
        width = height * target_width / target_height;
    } else {
        height = width * target_height / target_width;
    }
    ensure!(width > 0 && height > 0, "invalid camera crop");
    let x = (frame.width - width) / 2;
    let y = (frame.height - height) / 2;
    let mut bytes = Vec::with_capacity(usize::try_from(width * height)? * 4);
    for line in y..y + height {
        let start = usize::try_from((line * frame.width + x) * 4)?;
        bytes.extend_from_slice(&frame.bytes[start..start + usize::try_from(width)? * 4]);
    }
    Ok(VideoFrame {
        width,
        height,
        bytes,
    })
}
