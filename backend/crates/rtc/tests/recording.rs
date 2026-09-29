use gst::prelude::*;
use gstreamer as gst;
use mahfel_domain::{ActorId, ProducerSource};
use mahfel_rtc::{
    media_codecs,
    recording::{CaptureClock, RecordingTap},
};
use mediasoup::{
    prelude::*, router::RouterOptions, worker::WorkerSettings, worker_manager::WorkerManager,
};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

fn listen() -> ListenInfo {
    ListenInfo {
        protocol: Protocol::Udp,
        ip: std::net::Ipv4Addr::LOCALHOST.into(),
        announced_address: None,
        expose_internal_ip: false,
        port: None,
        port_range: None,
        flags: None,
        send_buffer_size: None,
        recv_buffer_size: None,
    }
}

struct Fixture {
    source: gst::Pipeline,
    tap: RecordingTap,
    _producer: Producer,
    _transport: PlainTransport,
    directory: PathBuf,
    timestamps: Arc<Mutex<Option<(u64, u64)>>>,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = self.source.set_state(gst::State::Null);
    }
}

async fn fixture(router: &Router, clock: &CaptureClock, index: u32, codec: &str) -> Fixture {
    let mut options = PlainTransportOptions::new(listen());
    options.rtcp_mux = false;
    options.comedia = true;
    let transport = router
        .create_plain_transport(options)
        .await
        .expect("input transport");
    let (kind, mime, rate, payload, source, encode) = match codec {
        "opus" => (
            MediaKind::Audio,
            "audio/opus",
            48_000,
            100,
            ProducerSource::Microphone,
            "audiotestsrc is-live=true wave=sine ! audioconvert ! audioresample ! opusenc ! rtpopuspay",
        ),
        "vp8" => (
            MediaKind::Video,
            "video/VP8",
            90_000,
            101,
            ProducerSource::Camera,
            "videotestsrc is-live=true pattern=ball ! video/x-raw,width=320,height=180,framerate=15/1 ! vp8enc deadline=1 keyframe-max-dist=30 ! rtpvp8pay",
        ),
        "h264" => (
            MediaKind::Video,
            "video/H264",
            90_000,
            103,
            ProducerSource::Screen,
            "videotestsrc is-live=true pattern=ball ! video/x-raw,width=320,height=180,framerate=15/1 ! x264enc tune=zerolatency key-int-max=30 ! video/x-h264,profile=constrained-baseline ! rtph264pay config-interval=-1",
        ),
        _ => panic!("unknown fixture codec"),
    };
    let ssrc = 1000 + index;
    let mut codec_parameters = serde_json::json!({"mimeType":mime,"payloadType":payload,"clockRate":rate,"parameters":{},"rtcpFeedback":[]});
    if kind == MediaKind::Audio {
        codec_parameters["channels"] = serde_json::json!(2);
    }
    if codec == "h264" {
        codec_parameters["parameters"] = serde_json::json!({"packetization-mode":1,"profile-level-id":"42e01f","level-asymmetry-allowed":1});
    }
    let parameters = serde_json::from_value(serde_json::json!({
        "codecs":[codec_parameters],"headerExtensions":[],"encodings":[{"ssrc":ssrc}],"rtcp":{"cname":"recording-fixture","reducedSize":true}
    })).expect("input RTP");
    let producer = transport
        .produce(ProducerOptions::new(kind, parameters))
        .await
        .expect("input producer");
    let directory = std::env::var_os("MAHFEL_RECORDING_CRASH_DIRECTORY").map_or_else(
        || std::env::temp_dir().join(format!("mahfel-capture-{}", uuid::Uuid::new_v4())),
        |root| PathBuf::from(root).join(index.to_string()),
    );
    let tap = RecordingTap::create(
        router,
        &producer.id().to_string(),
        ActorId::new(),
        source,
        &directory,
        clock,
    )
    .await
    .expect("recording tap");
    let encode = encode.replace(" ! rtp", " ! identity name=encoded ! rtp");
    let description = format!(
        "rtpbin name=sender {encode} pt={payload} ssrc={ssrc} ! sender.send_rtp_sink_0 \
         sender.send_rtp_src_0 ! udpsink host=127.0.0.1 port={} sync=false async=false \
         sender.send_rtcp_src_0 ! udpsink host=127.0.0.1 port={} sync=false async=false",
        transport.tuple().local_port(),
        transport.rtcp_tuple().expect("RTCP tuple").local_port()
    );
    let source = gst::parse::launch(&description)
        .expect("source pipeline")
        .downcast::<gst::Pipeline>()
        .expect("source pipeline type");
    let timestamps = Arc::new(Mutex::new(None));
    let observed = Arc::clone(&timestamps);
    source
        .by_name("encoded")
        .expect("encoded fixture")
        .static_pad("src")
        .expect("encoded pad")
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
            {
                let mut bounds = observed.lock().expect("source timestamp lock");
                let first = bounds.map_or(pts.nseconds(), |(first, _)| first);
                *bounds = Some((first, pts.nseconds()));
            }
            gst::PadProbeReturn::Ok
        });
    clock.configure(&source);
    source
        .set_state(gst::State::Playing)
        .expect("source playing");
    Fixture {
        source,
        tap,
        _producer: producer,
        _transport: transport,
        directory,
        timestamps,
    }
}

async fn run(tracks: u32, seconds: u64) {
    let clock = CaptureClock::new().expect("capture clock");
    let manager = WorkerManager::new();
    let worker = manager
        .create_worker(WorkerSettings::default())
        .await
        .expect("native worker");
    let router = worker
        .create_router(RouterOptions::new(media_codecs()))
        .await
        .expect("router");
    let mut fixtures = Vec::new();
    for index in 0..tracks {
        fixtures.push(
            fixture(
                &router,
                &clock,
                index,
                ["opus", "vp8", "h264"][index as usize % 3],
            )
            .await,
        );
    }
    let start = Instant::now();
    let mut last_keyframe = Instant::now();
    let mut fragments = vec![Vec::new(); tracks as usize];
    let mut reported = false;
    while start.elapsed() < Duration::from_secs(seconds) {
        tokio::time::sleep(Duration::from_millis(100)).await;
        for (index, fixture) in fixtures.iter().enumerate() {
            if let Some(message) = fixture
                .source
                .bus()
                .expect("source bus")
                .pop_filtered(&[gst::MessageType::Error])
            {
                panic!("fixture failure: {message:?}");
            }
            if last_keyframe.elapsed() >= Duration::from_secs(5) {
                fixture
                    .tap
                    .request_key_frame()
                    .await
                    .expect("request keyframe");
            }
            fragments[index].extend(fixture.tap.pipeline.drain().expect("capture bus"));
        }
        if last_keyframe.elapsed() >= Duration::from_secs(5) {
            last_keyframe = Instant::now();
        }
        if !reported
            && fragments.iter().all(|segments| !segments.is_empty())
            && let Some(root) = std::env::var_os("MAHFEL_RECORDING_CRASH_DIRECTORY")
        {
            let report: Vec<_> = fragments
                .iter()
                .flatten()
                .map(|segment| {
                    serde_json::json!({
                        "path": segment.path, "start_ns": segment.start_ns, "end_ns": segment.end_ns
                    })
                })
                .collect();
            std::fs::write(
                PathBuf::from(root).join("sealed.json"),
                serde_json::to_vec(&report).expect("crash report JSON"),
            )
            .expect("crash report");
            reported = true;
        }
    }
    for fixture in &fixtures {
        fixture
            .source
            .set_state(gst::State::Paused)
            .expect("freeze fixture source");
    }
    tokio::time::sleep(Duration::from_millis(500)).await;
    let mut timing = Vec::new();
    for (index, fixture) in fixtures.iter().enumerate() {
        let received = fixture
            .tap
            .pipeline
            .media_bounds()
            .expect("captured bounds")
            .expect("captured samples");
        let sent = fixture
            .timestamps
            .lock()
            .expect("sent bounds lock")
            .expect("sent samples");
        let drift = (received.1 - received.0).abs_diff(sent.1 - sent.0);
        assert!(
            drift <= 100_000_000,
            "track {index} duration drift {} ms, received={received:?} sent={sent:?}",
            drift / 1_000_000
        );
        timing.push((received, sent, drift));
        fixture.tap.seal().await.expect("seal capture");
    }
    let offsets: Vec<i128> = timing
        .iter()
        .map(|(received, sent, _)| i128::from(received.0) - i128::from(sent.0))
        .collect();
    let skew = offsets.iter().max().expect("offset maximum")
        - offsets.iter().min().expect("offset minimum");
    assert!(
        skew <= 100_000_000,
        "cross-track start skew {} ms",
        skew / 1_000_000
    );
    println!("cross_track_start_skew_ns={skew}");
    let finalization = Instant::now();
    let mut bytes = 0;
    tokio::time::sleep(Duration::from_secs(1)).await;
    for (index, fixture) in fixtures.iter().enumerate() {
        fragments[index].extend(fixture.tap.pipeline.drain().expect("sealed fragments"));
        assert!(
            !fragments[index].is_empty(),
            "track {index} produced no fragments"
        );
        for segment in &fragments[index] {
            assert!(segment.path.metadata().expect("fragment file").len() > 100);
            bytes += segment.path.metadata().expect("fragment size").len();
            assert!(
                segment.end_ns > segment.start_ns,
                "positive segment duration"
            );
            verify_decode(&segment.path);
        }
        println!(
            "track={index} codec={} fragments={} first_ns={} last_ns={}",
            fixture.tap.pipeline.codec,
            fragments[index].len(),
            fragments[index][0].start_ns,
            fragments[index].last().expect("last").end_ns
        );
        println!("timing={:?}", timing[index]);
        std::fs::remove_dir_all(&fixture.directory).expect("remove fixture fragments");
    }
    println!(
        "tracks={tracks} seconds={seconds} bytes={bytes} bytes_per_second={} decode_verification_ms={}",
        bytes / seconds,
        finalization.elapsed().as_millis()
    );
}

#[tokio::test]
async fn captures_and_decodes_all_router_codecs() {
    run(3, 8).await;
}

#[tokio::test]
#[ignore = "ten-minute native RTP recording/resource benchmark"]
async fn recording_resource_profile() {
    let tracks = std::env::var("MAHFEL_RECORDING_BENCH_TRACKS")
        .unwrap_or_else(|_| "8".into())
        .parse()
        .expect("track count");
    let seconds = std::env::var("MAHFEL_RECORDING_BENCH_SECONDS")
        .unwrap_or_else(|_| "600".into())
        .parse()
        .expect("duration");
    run(tracks, seconds).await;
}

fn verify_decode(path: &std::path::Path) {
    gst::init().expect("GStreamer initialization");
    let decode = gst::parse::launch("uridecodebin name=input ! fakesink sync=false")
        .expect("decode pipeline")
        .downcast::<gst::Pipeline>()
        .expect("decode pipeline type");
    decode
        .by_name("input")
        .expect("decode source")
        .set_property("uri", format!("file://{}", path.display()));
    decode.set_state(gst::State::Playing).expect("decode start");
    let message = decode.bus().expect("decode bus").timed_pop_filtered(
        gst::ClockTime::from_seconds(10),
        &[gst::MessageType::Eos, gst::MessageType::Error],
    );
    decode.set_state(gst::State::Null).expect("decode stop");
    assert!(
        message.is_some_and(|message| message.type_() == gst::MessageType::Eos),
        "fragment decodes to EOS: {}",
        path.display()
    );
}

#[tokio::test]
#[ignore = "subprocess fixture for recording restart test"]
async fn recording_crash_child() {
    assert!(std::env::var_os("MAHFEL_RECORDING_CRASH_DIRECTORY").is_some());
    run(3, 600).await;
}

#[tokio::test]
async fn finalized_fragments_survive_process_kill() {
    let root = std::env::temp_dir().join(format!("mahfel-crash-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).expect("crash fixture directory");
    let mut child = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args(["recording_crash_child", "--exact", "--ignored"])
        .env("MAHFEL_RECORDING_CRASH_DIRECTORY", &root)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("crash subprocess");
    let start = Instant::now();
    while !root.join("sealed.json").exists() && start.elapsed() < Duration::from_secs(20) {
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(
            child.try_wait().expect("subprocess status").is_none(),
            "fixture terminated early"
        );
    }
    child.kill().expect("kill active recording process");
    child.wait().expect("reap recording process");
    let report: Vec<serde_json::Value> = serde_json::from_slice(
        &std::fs::read(root.join("sealed.json")).expect("sealed fragment report"),
    )
    .expect("report JSON");
    assert!(report.len() >= 3);
    let elapsed_ns = start.elapsed().as_nanos();
    let mut max_loss_ns = 0;
    for segment in &report {
        verify_decode(&PathBuf::from(
            segment["path"].as_str().expect("fragment path"),
        ));
        let loss = elapsed_ns.saturating_sub(u128::from(
            segment["end_ns"].as_u64().expect("end timestamp"),
        ));
        max_loss_ns = max_loss_ns.max(loss);
    }
    assert!(
        max_loss_ns < 10_000_000_000,
        "restart loss exceeds fixture GOP/fragment window"
    );
    println!(
        "killed_process_finalized_fragments={} max_unsealed_window_ns={max_loss_ns}",
        report.len()
    );
    std::fs::remove_dir_all(root).expect("remove restart fixture");
}

#[tokio::test]
async fn resumed_capture_preserves_the_pause_gap() {
    let clock = CaptureClock::new().expect("capture clock");
    let manager = WorkerManager::new();
    let worker = manager
        .create_worker(WorkerSettings::default())
        .await
        .expect("worker");
    let router = worker
        .create_router(RouterOptions::new(media_codecs()))
        .await
        .expect("router");
    let mut fixture = fixture(&router, &clock, 0, "opus").await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    fixture.tap.seal().await.expect("pause capture");
    tokio::time::sleep(Duration::from_millis(200)).await;
    let before = fixture.tap.pipeline.drain().expect("paused fragments");
    let end_ns = before.last().expect("pre-pause fragment").end_ns;
    let sent_before = fixture
        .timestamps
        .lock()
        .expect("source bounds")
        .expect("source samples")
        .1;
    tokio::time::sleep(Duration::from_secs(2)).await;
    let sent_after = fixture
        .timestamps
        .lock()
        .expect("source bounds")
        .expect("source samples")
        .1;
    assert!(
        sent_after > sent_before + 1_800_000_000,
        "publisher continues during recording pause"
    );
    let resumed_directory = fixture.directory.join("resume");
    let resumed = RecordingTap::create(
        &router,
        &fixture.tap.producer_id,
        fixture.tap.actor_id,
        fixture.tap.source,
        &resumed_directory,
        &clock,
    )
    .await
    .expect("resume tap");
    let paused = std::mem::replace(&mut fixture.tap, resumed);
    tokio::time::sleep(Duration::from_secs(2)).await;
    fixture.tap.seal().await.expect("seal resumed capture");
    tokio::time::sleep(Duration::from_millis(200)).await;
    let after = fixture.tap.pipeline.drain().expect("resumed fragments");
    let start_ns = after.first().expect("post-pause fragment").start_ns;
    assert!(
        start_ns > end_ns + 1_800_000_000,
        "pause must remain in the shared timeline"
    );
    for segment in before.iter().chain(&after) {
        verify_decode(&segment.path);
    }
    println!("preserved_pause_gap_ns={}", start_ns - end_ns);
    drop(paused);
    std::fs::remove_dir_all(&fixture.directory).expect("remove pause fixture");
}
