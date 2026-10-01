# Original-track session recordings

M9-01 through M9-05 capture original media and a replay bundle; M9-06 adds authorized interactive
playback and retention deletion. The media-node capture pipeline retains original codecs without
transcoding or composition. The worker separately prepares browser-compatible, per-source MP4
segments for interactive replay. Optional composite MP4 export is a separate capability in
[ADR 0028](../adr/0028-recording-mp4-export.md).

## Controls and durable state

Sessions default to `auto_record = true`. Managers may change the preference while scheduled;
starting a session with the preference enabled first obtains a 15-second PostgreSQL start lease.
The selected media node must confirm capture codecs, room creation, spool capacity and a private S3 write/read/delete
before the session commits Live and creates its recording intent in the same transaction. A failed
preflight leaves the session Scheduled, so the manager can retry. A crashed request's lease expires.
The preflight deadline is eight seconds, leaving time to return an actionable failure before the
browser request times out.
Sessions with `auto_record = false` start without that preflight. Existing live sessions are not
automatically started by migration. Capture or storage failure after a successful start leaves the
session live with a recording warning and an explicit coverage gap.

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
visible coverage warning. Session end returns after committing the terminal boundary. Background
cleanup waits up to 30 seconds for sealing; a deadline marks remaining capture as lost before live
media is released. The media reconciler also observes the durable terminal state after API restart.

## Capture, upload and recovery

The private `ReconcileRecording` RPC carries only a fenced capture intent and bounded status. The
owning media node observes dynamic producers, including reconnects, and captures one original
stream per microphone/camera/screen producer. Video selects spatial layer zero when simulcast is
available. RTP/RTCP stay on loopback plain transports, with RTCP receiver feedback returned to the
SFU. Track timing uses the shared monotonic receive clock; late RTCP sender reports do not reset
the mux timeline. Original Opus/VP8 are muxed into WebM; H264 is muxed into Matroska. Keyframe requests target
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
recording. Each verified fragment is also converted to a standalone H264 or AAC MP4 segment,
preserving its source identity and session-relative clock bounds in a separate content-addressed
playback index. No usable replay data produces failed state. Storage or conversion failures retry;
the recording is not Ready until the original manifest and browser index are both accepted under
the finalization claim. A dead finalization job shows a replay preparation failure, which a manager
can retry through the recording UI. Existing Ready recordings without an index are queued for
backfill by migration 0033.
Preparation enforces the same 8 MiB index and 10,000-segment limits as playback; exceeding either
limit fails preparation rather than publishing an unreadable Ready recording. Derived video is
normalized to 30 fps to provide continuous MP4 presentation timestamps.

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
compressed. Original fragments remain immutable replay assets; derived browser segments live under
the same recording prefix and are deleted by retention with the original bundle.

PostgreSQL accepts one manifest atomically only while the finalization claim/lease and sealed
capture generation still match. A lost claim cannot accept stale output. Duplicate retries reuse an
accepted result. Content-addressed temporary manifest attempts are not user-visible artifacts;
retention removes them through the deferred cleanup described below.

## Playback and retention

`GET /v1/sessions/{id}/recordings/{recording_id}/playback` is authorized by the same
`manage_recording`/membership rule as export download: current room members or administrators only,
with active blocks enforced. Public room visibility alone grants nothing. The response is a replay
descriptor: `manifest_key`, a map of short-lived presigned object URLs for the manifest and its
content-addressed pages, a `file_id`-to-URL map for archived presentation assets, and media track
metadata from the versioned playback index. The browser reconstructs `ReplayInput` from those
pages and reuses the replay engine. It requests at most a 30-second media URL window from
`GET /v1/sessions/{id}/recordings/{recording_id}/playback/media-window?start_ms=...`, refreshing
short-lived URLs as playback or seeking moves. Separate camera, screen and microphone elements
follow the same session clock; unavailable intervals remain visible gaps. No media bandwidth is
proxied through the API. Unauthorized callers receive `404` with no URL.

Managers with `manage_recording` and session access may delete a recording with
`DELETE /v1/sessions/{id}/recordings/{recording_id}`. Deletion is durable and audited
(`recording_audit.action_kind = 'delete'`): the recording is soft-deleted, marked `failed`, its
revision advances so stale export acceptance is fenced, and pending archive pins are released.
Deleted recordings disappear from listings and can no longer be replayed or downloaded.

Retention cleanup runs on the worker's five-minute sweep. For each soft-deleted recording it deletes
every object under `recordings/{recording}/` (manifest pages, fragments, archived assets and any
generated export) and only then removes the corresponding `recording_fragments`, `recording_events`,
`recording_snapshots`, `recording_assets`, `recording_exports` and `recording_export_attempts` rows
and stamps `cleanup_checked_at`. An incomplete object deletion is retried on the next sweep rather
than silently finalizing; immutable audit history is retained.

## Validation and monitoring

### Local completion evidence (2026-09-30)

The final API, media, worker and web images built and ran in the local Compose stack.
The backend unit suites passed 390 tests; frontend suites passed 152 tests, with
the final affected components also checked separately. Strict Clippy, SQL lint,
Rust formatting, frontend lint/typecheck, protobuf checks and documentation checks passed.
Real PostgreSQL tests covered start leases, preference races, finalization fencing,
legacy playback backfill, export deduplication, authorization and retention. The worker
integration used PostgreSQL, RabbitMQ and RustFS to verify retry, original fragments,
derived browser media, archived assets and cleanup. Native Opus/VP8/H264 capture checks passed.

Authenticated Chromium capture produced microphone, camera, screen and notes recordings.
Interactive replay passed in Chromium and Firefox, including advancing media, pause,
microphone mute, seeking and notes reconstruction. An unrelated account received 404
for playback descriptors, media windows and export downloads. The current worker produced
an accepted MP4 that downloaded at the recorded byte size and decoded both streams:
H264 1920x1080 video and 48 kHz stereo AAC audio, with a 13.04-second duration.

Pausing local RustFS made recorded startup return 503 in 8.04 seconds while the session
remained Scheduled. An explicitly unrecorded session started during the outage; retrying
recorded startup after storage recovery succeeded. Storage was restored after the drill.

These results establish local runtime behavior; they do not establish CI or production
deployment. Firefox replay used Chromium-captured media: Firefox live publication remains
tracked under TASK-20. The stale media drain flag after a quick local restart remains
tracked under TASK-83; the local verification cleared that ephemeral flag explicitly.

See the [native spike evidence](../development/recording-spike.md) for ten-minute clock checks and
1/8/24-track resource profiles. Run targeted gates from `backend/`:

```bash
cargo test -p mahfel-rtc -p mahfel-media
MAHFEL_TESTCONTAINERS=1 cargo test -p mahfel-postgres --test recordings
MAHFEL_TESTCONTAINERS=1 cargo test -p mahfel-worker --bin mahfel-worker
```

The PostgreSQL integration covers policy, UUID deduplication, event exclusion, resume baselines,
archive pins, boot/revision fencing, atomic manifest acceptance and retention deletion (soft-delete,
cleanup finalization, removed bundle/export rows, audit). `mahfel-application` unit tests prove an
authorized member receives replay URLs while an unauthorized caller receives no URL. Media tests
cover sealing, stale commands and journal recovery after upload failure/restart. The worker
integration runs PostgreSQL, RustFS and RabbitMQ with an original Opus fragment, storage retry,
corruption/partial coverage, archive verification and duplicate finalization.

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

## Native export renderer validation, 2026-10-01

[ADR 0030](../adr/0030-native-recording-image-rendering.md) replaces the historical Chromium
export packaging described in the September validation evidence. The worker requests native images
from the authenticated Next.js endpoint on the existing web port; the web container runs Bun.

The production web and worker images build successfully. Worker executable/artifact checks show no
Bun, Node, Chromium or standalone frontend renderer. Thirteen endpoint tests cover authentication,
PNG/JPEG/SVG, resource limits, archived PDF/image verification, native drawing, Persian notes and
empty overlays. A rendered production-container PNG was visually checked for shaped Persian glyphs
using the private TTF font. Four Rust replay regressions verify generations, backward seeking,
presentation timing, chat changes, outage/pause policy and camera/screen layout. The real native MP4
fixture passed against the production Bun/Next.js web image, with original camera/audio, archived
presentations and rich Persian notes, and both output streams decoded to EOS.

These are local build/runtime checks. They do not establish production deployment or rerun browser
login/E2E evidence. Production configuration requires matching rendering keys, an internal worker
renderer URL and an explicit web storage-origin allowlist; see the
[export operations contract](../development/recording-export.md).
