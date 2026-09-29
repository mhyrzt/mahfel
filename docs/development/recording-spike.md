# Native recording spike

The M9-01 prototype is in `mahfel-rtc::recording`. It consumes original encoded media through
loopback mediasoup plain transports and GStreamer RTP depayloaders, then writes per-track WebM
(Opus/VP8) or Matroska (H264) fragments. The product capture pipeline has no encoder or compositor.
Encoders appear only in the synthetic integration fixtures.

## Host dependencies

On Fedora, install the development libraries in addition to the native mediasoup toolchain:

```bash
sudo dnf install gstreamer1-devel gstreamer1-plugins-base-devel \
  gstreamer1-plugins-base gstreamer1-plugins-good gstreamer1-plugins-bad-free
```

The fixture also needs `opusenc`, `vp8enc`, `x264enc`, and a decoder for each codec. Check the
plugins with `gst-inspect-1.0`. The CI integration image installs these fixture dependencies.
Debian builds use `libgstreamer1.0-dev` and `libgstreamer-plugins-base1.0-dev`; the media runtime
image includes the base, good and bad plugin sets. GStreamer 1.22 or later is required.

## Reproduce

From `backend/`:

```bash
cargo test -p mahfel-rtc --test recording -- --nocapture
MAHFEL_RECORDING_BENCH_TRACKS=3 MAHFEL_RECORDING_BENCH_SECONDS=600 \
  /usr/bin/time -v cargo test -p mahfel-rtc --test recording \
  recording_resource_profile -- --ignored --nocapture
```

For resource measurements, compile first and time the test executable printed by
`cargo test -p mahfel-rtc --test recording --no-run`, so build costs are excluded. Run the
ignored profile with track counts 1, 8 and 24. Set `TMPDIR` to a directory on the intended spool
filesystem; `/tmp` may be memory-backed and cannot establish physical disk performance.

The fixture cycles Opus, VP8 and H264, using 320×180 video at 15 fps, a two-second GOP and an Opus
sine wave. All pipelines share one monotonic clock/base time. Encoder PTS is converted through
its GStreamer segment to running time before comparison, including H264's encoder segment offset.
Assertions check cross-track offset spread and source/capture duration drift against 100 ms,
positive fragment bounds, nonempty bytes, and actual decoding to EOS. These are controlled
loopback measurements, not proof of arbitrary browser clocks or network conditions.

`finalized_fragments_survive_process_kill` kills a subprocess without graceful EOS and verifies
that the reported finalized fragments still decode. The active fragment is deliberately excluded
from accepted output. `resumed_capture_preserves_the_pause_gap` keeps the publisher running while
capture is paused and verifies that a fresh tap retains the pause in the shared timeline.

## Interpretation

CPU and RSS include the source encoders, mediasoup worker, RTP capture and serial decode checks.
They do not isolate production capture cost or establish an admission limit. Byte rate measures
container output, not S3 upload performance. Temporary spool is not durable business state:
process restart can retain finalized files on the same filesystem, while host/ephemeral-volume
loss also loses all not-yet-uploaded fragments. Five seconds is a fragment target, not a maximum
loss guarantee: video fragmentation waits for a keyframe, and an absent publisher keyframe can
extend the current fragment. Production capture must enforce spool limits and surface gaps.

## Measured resource profile (2026-09-28)

Fedora host, Intel Core i7-10610U (four cores/eight logical CPUs), GStreamer 1.26.11,
Rust debug build, 30-second capture runs on the repository's disk filesystem. CPU is percent
of one logical CPU over capture and verification; peak RSS is the test process's maximum
reported by `/usr/bin/time -v`. The subprocess worker is included in CPU totals but its RSS
is not added to that peak. Runs include the synthetic encoders and serial fragment decoding.

| Tracks | CPU | Peak RSS (MiB) | Container bytes | Bytes/s | Initial skew (ms) | Max duration drift (ms) | Seal wait + decode checks (ms) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 3% | 50.1 | 300240 | 10008 | 0.000 | 1.440 | 1141 |
| 8 | 26% | 258.7 | 1774949 | 59164 | 1.561 | 3.996 | 2310 |
| 24 | 76% | 477.5 | 5337354 | 177911 | 3.852 | 5.347 | 5583 |

The 600-second three-track fixture produced 421 decodable fragments (13,089,653 bytes),
2.542 ms initial skew and 5.090 ms maximum duration drift. Its complete test took 610.85 seconds,
including 10.45 seconds of seal wait/decode verification, with 15% CPU and 173.8 MiB peak process
RSS. This longer run used `/tmp` (tmpfs), so its byte rate is not a physical disk benchmark.

An earlier 24-track disk run measured 134 ms at a live endpoint while a native rebuild competed
for CPU. The corrected harness freezes all fixture sources and waits 500 ms for RTP to drain
before comparing source/capture endpoints; all repeated 1/8/24 disk runs passed. The failed run
is not evidence of a supported production capacity. Capacity/admission and independent device
clock behavior still need deployment-specific measurements.
