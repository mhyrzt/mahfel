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
