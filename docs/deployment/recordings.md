# Original-track session recordings

M9-01 through M9-05 capture original media and a replay bundle. Playback and retention are M9-06.
The capture pipeline does not reconstruct video, transcode, composite, generate MP4 or use FFmpeg.
GStreamer decoders run only during finalization verification and test fixtures.

## Controls and durable state

Sessions default to `auto_record = true`. Managers may change the preference while scheduled;
starting a session with the preference enabled creates a recording intent in the same transaction.
Existing live sessions are not automatically started by migration. Recording startup or storage
failure leaves the session live and exposes a recording warning.

Managers, administrators and moderators with access to the target session have `manage_recording`.
Members see status and warnings but cannot issue commands. The API independently authorizes every
request. `GET /v1/sessions/{id}/recordings` lists metadata; `POST /v1/sessions/{id}/recordings/commands`
accepts a command UUID and `start`, `pause`, `resume` or `stop`. A repeated UUID replays its original
result; reuse for another actor/action conflicts. Commands serialize with the session write boundary.
The browser polls authoritative state every three seconds. A compact toolbar indicator is visible
to every participant; its upward-opening menu shows status, capture details and warnings. Only
authorized users see Start/Pause/Resume/Stop menu actions.

Start creates a new recording. Pause excludes subsequent workspace and session-chat events and
requests sealing of the current media generation. Resume uses the same recording with a new capture
generation and a fresh current-workspace snapshot. Paused edits appear in that snapshot, while
paused events and chat messages remain excluded. Edits/deletes of recorded chat messages retain
history; messages created during a pause cannot reappear through a later edit. Stop finalizes the
recording; a later Start creates a different recording.

PostgreSQL owns recording state, audit, generation intervals, snapshots, event history, fragment
receipts, archive-copy metadata and the accepted manifest. Media nodes hold temporary taps/spools.
RabbitMQ delivers finalization jobs and never owns recording truth. Media boot IDs, generations and
command revisions fence stale reports. A restarted media node creates a new generation and a
visible coverage warning. Session end commits the terminal boundary before waiting up to 30 seconds
for sealing; a deadline marks remaining capture as lost before live media is released.

## Capture, upload and recovery

The private `ReconcileRecording` RPC carries only a fenced capture intent and bounded status. The
owning media node observes dynamic producers, including reconnects, and captures one original
stream per microphone/camera/screen producer. Video selects spatial layer zero when simulcast is
available. RTP/RTCP stay on loopback plain transports, with RTCP receiver feedback returned to the
SFU. Original Opus/VP8 are muxed into WebM; H264 is muxed into Matroska. Keyframe requests target
five-second fragments. Unsupported codecs or failed taps produce an explicit warning and capped
30-second exponential recovery; publishing continues.

Sealed fragments have SHA-256, original clock origin and running-time bounds. Atomic spool journals
are retained until both the fragment and its immutable receipt are confirmed in private S3.
Restart with the same spool filesystem retries journaled uploads. Active unsealed fragments and
loss of the spool volume are coverage losses, not silently accepted replay data. The API and
finalizer import S3 receipts idempotently. Video fragmentation depends on keyframes, so five seconds
is a target rather than a guaranteed crash-loss bound.

`backend/config/default.toml` exposes these settings (environment overrides use double underscores):

| Setting | Default |
| --- | --- |
| `recording.spool_directory` | `/tmp/mahfel-recordings` |
| `recording.recording_spool_bytes` | 268435456 (256 MiB) |
| `recording.node_spool_bytes` | 1073741824 (1 GiB) |
| `recording.upload_concurrency` | 4 (maximum 4) |

The coordinator checks open and pending files every 250 ms and stops capture at spool pressure.
In-flight mux/queue bytes can finish while sealing; provision filesystem headroom beyond the
threshold. Encoded queues are bounded to 4 MiB per tap. Individual accepted fragments are capped
at 32 MiB. Storage failure keeps journals for retry; capacity exhaustion never blocks publishers.
Compose mounts a named spool volume; Minikube mounts a disk-backed 2 GiB emptyDir, preserving
container restarts within a pod but not pod/host loss. PostgreSQL and S3 remain authoritative.
API, media and worker must use the same private bucket and storage credentials.

Referenced presentation files, embedded images and recorded chat attachments receive private
`recordings/{recording}/assets/{file}` copies. Pending copies pin source bytes and metadata against
physical cleanup; logical deletion may proceed when live collaboration references permit it.
The API verifies archive checksums before releasing pins. The finalizer also copies/verifies pending
assets. An accepted archive survives later source deletion. Failed jobs retain pending source pins
until operator replay or an explicit future retention operation resolves them.

## Finalization and replay contract

`recordings.finalize.v1` uses the [background-job](background-jobs.md) transactional outbox, quorum
queue, claim renewal, bounded retries, DLQ and operator replay. Only sealed generations finalize.
The worker verifies original SHA-256/byte counts and decodes each container to EOS. Corrupt fragments
are excluded with explicit losses; usable remaining media/workspace/chat produces a partial ready
recording. No usable replay data produces failed state. Storage failures retry and exhaustion marks
finalization failed. Operator replay restores finalizing state for the same sealed capture.

The version-one manifest uses content-addressed JSON pages of at most 100 source rows for
`generations`, `snapshots`, `events`, `assets` and verified `fragments`. It includes track identities,
actor/source/codec/container metadata, source clocks, active intervals, pause/media gaps and losses.
Generation intervals with capture failures are explicitly marked as uncertain coverage, even when
exact missing-media bounds are unavailable.
Snapshot byte arrays use PostgreSQL hexadecimal bytea encoding. A document baseline consists of the
latest checkpoint plus later document updates captured at the boundary. Current presentation,
viewport, grants and embedded-asset associations accompany the baseline. Original event sequence
numbers and timestamps preserve ordering; awareness is excluded.

`recording.session_origin_ns` is the session's start epoch in nanoseconds. A fragment timestamp maps
to session time as `unix_origin_ns + running_time_ns - session_origin_ns`. Event timestamps and
capture-generation boundaries map to that same origin. Consumers must restrict playback to active
generation intervals, including the terminal boundary; asynchronous sealing can leave an original
fragment extending past a requested Pause/Stop boundary. Paused/outage time is preserved, never
compressed. Original fragments remain replay assets; finalization never overwrites or exports them.

PostgreSQL accepts one manifest atomically only while the finalization claim/lease and sealed
capture generation still match. A lost claim cannot accept stale output. Duplicate retries reuse an
accepted result. Content-addressed temporary manifest attempts are not user-visible artifacts;
retention/garbage collection is intentionally left to M9-06 and later work.

## Validation and monitoring

See the [native spike evidence](../development/recording-spike.md) for ten-minute clock checks and
1/8/24-track resource profiles. Run targeted gates from `backend/`:

```bash
cargo test -p mahfel-rtc -p mahfel-media
MAHFEL_TESTCONTAINERS=1 cargo test -p mahfel-postgres --test recordings
MAHFEL_TESTCONTAINERS=1 cargo test -p mahfel-worker --bin mahfel-worker
```

The PostgreSQL integration covers policy, UUID deduplication, event exclusion, resume baselines,
archive pins, boot/revision fencing and atomic manifest acceptance. Media tests cover sealing,
stale commands and journal recovery after upload failure/restart. The worker integration runs
PostgreSQL, RustFS and RabbitMQ with an original Opus fragment, storage retry, corruption/partial
coverage, archive verification and duplicate finalization.

Frontend Storybook includes `Recordings/Toolbar Menu`, `Recordings/Scheduled Preference`
and recording variants under `Sessions/Session Screen`. These use the production view with local
UI fixtures and need no API/media connection. They cover lifecycle controls, participant notices,
loading/pending/failure states, scheduled-only preferences, Persian/RTL and narrow layouts.
`Interactive Lifecycle` exercises Start/Pause/Resume/Stop. From `frontend/`, run
`bun run storybook` to explore or `bun run storybook:build` for the static build.

Media metrics include `mahfel_recording_active_generations`, `mahfel_recording_spool_bytes`,
`mahfel_recording_failures_total{reason}` and `mahfel_recording_upload_cycles_total{outcome}`.
Labels use bounded reasons/outcomes. IDs appear only in structured logs. Worker job metrics and
inspection/replay commands are described in the background-job runbook. Native fixture checks do
not establish arbitrary browser-device clock accuracy or deployment-specific recording capacity.
