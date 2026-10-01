use gstreamer::{self as gst, prelude::*};
use mahfel_application::ports::{BoxFuture, RecordingVerifier, StoreError};

pub struct GstreamerVerifier;
impl RecordingVerifier for GstreamerVerifier {
    fn readable<'a>(
        &'a self,
        bytes: Vec<u8>,
        container: &'a str,
    ) -> BoxFuture<'a, Result<bool, StoreError>> {
        Box::pin(async move {
            if !matches!(container, "webm" | "mkv") {
                return Ok(false);
            }
            let container = container.to_owned();
            tokio::task::spawn_blocking(move || verify(bytes, &container))
                .await
                .map_err(|error| StoreError::Unexpected(error.to_string()))?
        })
    }
    fn browser_segment<'a>(
        &'a self,
        bytes: Vec<u8>,
        container: &'a str,
        audio: bool,
    ) -> BoxFuture<'a, Result<Vec<u8>, StoreError>> {
        Box::pin(async move {
            let container = container.to_owned();
            tokio::task::spawn_blocking(move || encode_browser_segment(bytes, &container, audio))
                .await
                .map_err(|error| StoreError::Unexpected(error.to_string()))?
        })
    }
}

fn encode_browser_segment(
    bytes: Vec<u8>,
    extension: &str,
    audio: bool,
) -> Result<Vec<u8>, StoreError> {
    gst::init().map_err(|error| StoreError::Unexpected(error.to_string()))?;
    let id = uuid::Uuid::new_v4();
    let input = Temporary(std::env::temp_dir().join(format!("mahfel-browser-{id}.{extension}")));
    let output = Temporary(std::env::temp_dir().join(format!("mahfel-browser-{id}.mp4")));
    std::fs::write(&input.0, bytes).map_err(StoreError::unexpected)?;
    let chain = if audio {
        "audio/x-raw ! audioconvert ! audioresample ! audio/x-raw,rate=48000,channels=2 ! avenc_aac bitrate=128000 ! aacparse"
    } else {
        "video/x-raw ! videoconvert ! videorate ! video/x-raw,format=I420,framerate=30/1 ! x264enc bitrate=4000 speed-preset=fast tune=zerolatency threads=2 key-int-max=60 ! h264parse"
    };
    let pipeline = gst::parse::launch(&format!(
        "uridecodebin uri=file://{} name=input input. ! queue ! {chain} ! mp4mux fragment-duration=5000 ! filesink location={}",
        input.0.display(), output.0.display(),
    ))
    .map_err(StoreError::unexpected)?
    .downcast::<gst::Pipeline>()
    .map_err(|_| StoreError::Unexpected("invalid browser segment pipeline".into()))?;
    pipeline
        .set_state(gst::State::Playing)
        .map_err(StoreError::unexpected)?;
    let result = pipeline.bus().and_then(|bus| {
        bus.timed_pop_filtered(
            gst::ClockTime::from_seconds(30),
            &[gst::MessageType::Eos, gst::MessageType::Error],
        )
    });
    let _ = pipeline.set_state(gst::State::Null);
    match result.as_ref().map(|message| message.view()) {
        Some(gst::MessageView::Eos(..)) => {}
        Some(gst::MessageView::Error(error)) => {
            return Err(StoreError::Unexpected(format!(
                "browser segment conversion failed: {} ({:?})",
                error.error(),
                error.debug()
            )));
        }
        _ => {
            return Err(StoreError::Unexpected(
                "browser segment conversion timed out".into(),
            ));
        }
    }
    let output = std::fs::read(&output.0).map_err(StoreError::unexpected)?;
    if output.is_empty() || output.len() > 64 * 1024 * 1024 {
        return Err(StoreError::Unexpected(
            "browser segment size outside bounds".into(),
        ));
    }
    Ok(output)
}
fn verify(bytes: Vec<u8>, extension: &str) -> Result<bool, StoreError> {
    gst::init().map_err(|error| StoreError::Unexpected(error.to_string()))?;
    let path = std::env::temp_dir().join(format!(
        "mahfel-verify-{}.{}",
        uuid::Uuid::new_v4(),
        extension
    ));
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| StoreError::Unexpected(error.to_string()))?;
    let temporary = Temporary(path);
    use std::io::Write;
    (&file)
        .write_all(&bytes)
        .map_err(|error| StoreError::Unexpected(error.to_string()))?;
    let pipeline = gst::parse::launch("uridecodebin name=input ! fakesink sync=false")
        .map_err(|error| StoreError::Unexpected(error.to_string()))?
        .downcast::<gst::Pipeline>()
        .map_err(|_| StoreError::Unexpected("invalid verification pipeline".into()))?;
    let source = pipeline
        .by_name("input")
        .ok_or_else(|| StoreError::Unexpected("missing verification source".into()))?;
    source.set_property("uri", format!("file://{}", temporary.0.display()));
    if pipeline.set_state(gst::State::Playing).is_err() {
        let _ = pipeline.set_state(gst::State::Null);
        return Ok(false);
    }
    let result = pipeline.bus().and_then(|bus| {
        bus.timed_pop_filtered(
            gst::ClockTime::from_seconds(10),
            &[gst::MessageType::Eos, gst::MessageType::Error],
        )
    });
    let _ = pipeline.set_state(gst::State::Null);
    Ok(result.is_some_and(|message| message.type_() == gst::MessageType::Eos))
}
struct Temporary(std::path::PathBuf);
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
