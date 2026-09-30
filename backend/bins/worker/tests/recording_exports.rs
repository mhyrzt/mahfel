use std::{error::Error, path::PathBuf, process::Command};

struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn renders_archived_workspace_and_native_media_to_verified_mp4() -> Result<(), Box<dyn Error>> {
    if std::env::var("MAHFEL_RECORDING_EXPORT_TEST").as_deref() != Ok("1") {
        eprintln!("MAHFEL_RECORDING_EXPORT_TEST=1 requires Bun, Chromium and GStreamer plugins");
        return Ok(());
    }
    let frontend = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../frontend");
    let fixture =
        Fixture(std::env::temp_dir().join(format!("mahfel-export-{}", uuid::Uuid::new_v4())));
    let generated = Command::new("bun")
        .current_dir(&frontend)
        .arg("scripts/recording-renderer/fixture.ts")
        .arg(&fixture.0)
        .status()?;
    assert!(generated.success(), "generate archived Yjs fixture");
    for (description, file) in [
        (
            "videotestsrc num-buffers=360 pattern=ball ! video/x-raw,width=320,height=240,framerate=30/1 ! vp8enc deadline=1 ! webmmux",
            "camera.webm",
        ),
        (
            "audiotestsrc num-buffers=600 samplesperbuffer=960 wave=sine ! audio/x-raw,rate=48000 ! opusenc ! matroskamux",
            "microphone.mka",
        ),
    ] {
        let generated = Command::new("gst-launch-1.0")
            .arg("-q")
            .args(description.split_whitespace())
            .args(["!", "filesink"])
            .arg(format!("location={}", fixture.0.join(file).display()))
            .status()?;
        assert!(generated.success(), "generate original media fragment");
    }
    let rendered = Command::new(env!("CARGO_BIN_EXE_mahfel-worker"))
        .arg("render-export")
        .arg(&fixture.0)
        .arg(frontend.join("recording-renderer-dist"))
        .status()?;
    assert!(
        rendered.success(),
        "render and decode both output streams to EOS"
    );
    assert!(std::fs::metadata(fixture.0.join("export.mp4"))?.len() > 10000);
    Ok(())
}
