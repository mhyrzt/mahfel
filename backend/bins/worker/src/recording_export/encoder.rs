use super::decoder::VideoFrame;
use anyhow::{Context, Result, ensure};
use gstreamer::{self as gst, prelude::*};
use std::path::Path;

pub(super) struct Encoder {
    pipeline: gst::Pipeline,
    overlay: gst::Element,
    audio: gst::Element,
    videos: Vec<gst::Element>,
    pads: Vec<gst::Pad>,
}
impl Encoder {
    pub(super) fn new(path: &Path, width: i32, height: i32) -> Result<Self> {
        let video_inputs = (0..10).map(|i| format!("appsrc name=video{i} format=time is-live=false block=true max-buffers=2 ! videoconvert ! mix.sink_{}", i + 1)).collect::<Vec<_>>().join(" ");
        let description = format!(
            "compositor name=mix background=black ! video/x-raw,width={width},height={height},framerate=30/1 ! videoconvert ! video/x-raw,format=I420 ! x264enc bitrate=4000 speed-preset=fast tune=zerolatency threads=2 key-int-max=60 ! h264parse ! queue max-size-time=2000000000 max-size-bytes=16777216 max-size-buffers=0 ! mp4mux name=mux faststart=true ! filesink name=file appsrc name=overlay caps=image/png,framerate=30/1 format=time is-live=false block=true max-buffers=2 ! pngdec ! videoconvert ! mix.sink_0 appsrc name=audio caps=audio/x-raw,format=F32LE,rate=48000,channels=2,layout=interleaved format=time is-live=false block=true max-buffers=2 ! audioconvert ! avenc_aac bitrate=128000 ! aacparse ! queue max-size-time=2000000000 max-size-bytes=1048576 max-size-buffers=0 ! mux.audio_0 {video_inputs}"
        );
        let pipeline = gst::parse::launch(&description)?
            .downcast::<gst::Pipeline>()
            .map_err(|_| anyhow::anyhow!("invalid encoder"))?;
        pipeline.by_name("mux").context("MP4 mux")?.set_property(
            "faststart-file",
            path.with_extension("mux.tmp")
                .to_str()
                .context("mux scratch path")?,
        );
        pipeline
            .by_name("file")
            .context("file sink")?
            .set_property("location", path.to_str().context("output path")?);
        let overlay = pipeline.by_name("overlay").context("overlay source")?;
        let audio = pipeline.by_name("audio").context("audio source")?;
        let mixer = pipeline.by_name("mix").context("compositor")?;
        mixer
            .static_pad("sink_0")
            .context("overlay pad")?
            .set_property("zorder", 20u32);
        let mut videos = Vec::new();
        let mut pads = Vec::new();
        for i in 0..10 {
            videos.push(
                pipeline
                    .by_name(&format!("video{i}"))
                    .context("video source")?,
            );
            pads.push(
                mixer
                    .static_pad(&format!("sink_{}", i + 1))
                    .context("video pad")?,
            );
        }
        pipeline.set_state(gst::State::Playing)?;
        Ok(Self {
            pipeline,
            overlay,
            audio,
            videos,
            pads,
        })
    }
    fn push(source: &gst::Element, bytes: Vec<u8>, pts: u64, duration: u64) -> Result<()> {
        let mut buffer = gst::Buffer::from_mut_slice(bytes);
        let writable = buffer.get_mut().context("writable buffer")?;
        writable.set_pts(gst::ClockTime::from_nseconds(pts));
        writable.set_duration(gst::ClockTime::from_nseconds(duration));
        source
            .emit_by_name::<gst::FlowReturn>("push-buffer", &[&buffer])
            .into_result()?;
        Ok(())
    }
    pub(super) fn frame(
        &self,
        png: Vec<u8>,
        videos: Vec<(VideoFrame, serde_json::Value)>,
        audio: Vec<f32>,
        pts: u64,
        duration: u64,
    ) -> Result<()> {
        Self::push(&self.overlay, png, pts, duration)?;
        for (index, (source, pad)) in self.videos.iter().zip(&self.pads).enumerate() {
            let video = videos.get(index);
            let (width, height, bytes) = video.map_or((1, 1, vec![0; 4]), |(frame, _)| {
                (frame.width, frame.height, frame.bytes.clone())
            });
            source.set_property(
                "caps",
                gst::Caps::builder("video/x-raw")
                    .field("format", "RGBA")
                    .field("width", width)
                    .field("height", height)
                    .field("framerate", gst::Fraction::new(30, 1))
                    .build(),
            );
            if let Some((_, layer)) = video {
                for name in ["x", "y", "width", "height"] {
                    let property = match name {
                        "x" => "xpos",
                        "y" => "ypos",
                        other => other,
                    };
                    pad.set_property(
                        property,
                        i32::try_from(layer[name].as_i64().context("scene geometry")?)?,
                    );
                }
                pad.set_property("alpha", 1.0f64);
                pad.set_property_from_str("sizing-policy", "keep-aspect-ratio");
                pad.set_property("zorder", 1u32);
            } else {
                pad.set_property("alpha", 0.0f64);
                pad.set_property("width", 1i32);
                pad.set_property("height", 1i32);
            }
            Self::push(source, bytes, pts, duration)?;
        }
        Self::push(
            &self.audio,
            audio.iter().flat_map(|value| value.to_le_bytes()).collect(),
            pts,
            duration,
        )?;
        if let Some(message) = self
            .pipeline
            .bus()
            .context("encoder bus")?
            .pop_filtered(&[gst::MessageType::Error])
            && let gst::MessageView::Error(error) = message.view()
        {
            anyhow::bail!("encode failed: {}", error.error());
        }
        Ok(())
    }
    pub(super) fn finish(&self) -> Result<()> {
        for source in [&self.overlay, &self.audio]
            .into_iter()
            .chain(self.videos.iter())
        {
            source
                .emit_by_name::<gst::FlowReturn>("end-of-stream", &[])
                .into_result()?;
        }
        let message = self
            .pipeline
            .bus()
            .context("encoder bus")?
            .timed_pop_filtered(
                gst::ClockTime::from_seconds(60),
                &[gst::MessageType::Eos, gst::MessageType::Error],
            )
            .context("encoder EOS timeout")?;
        ensure!(
            matches!(message.view(), gst::MessageView::Eos(_)),
            "encoder failed at EOS"
        );
        self.pipeline.set_state(gst::State::Null)?;
        Ok(())
    }
}
impl Drop for Encoder {
    fn drop(&mut self) {
        let _ = self.pipeline.set_state(gst::State::Null);
    }
}
