# ADR 0027: Record original tracks and a collaboration replay bundle

## Status

Accepted

## Context

M9 needs recordings of session media and shared workspace state. A mediasoup router forwards
encoded media; it does not produce a flattened recording. Recording must preserve live-session
availability and the deployment boundaries in ADRs 0003, 0004, 0005, 0008 and 0010. Recreating video,
transcoding, composition, MP4 generation and FFmpeg export are outside M9-01 through M9-05.

## Decision

Capture each published microphone, camera and screen producer using a dedicated, paused-first
mediasoup plain-transport consumer on the owning media node. GStreamer receives RTP and RTCP on
loopback, depayloads the original codec and muxes Opus/VP8 to WebM and H264 to Matroska. Capture does
not decode or encode the media. One selected simulcast layer is captured for each producer.

Use one monotonic clock/base time for all capture pipelines and preserve session-relative timing.
Target five-second independently readable fragments; video cuts depend on keyframes, requested
periodically. Track metadata retains producer, actor, source, codec, media-node boot identity,
capture generation, clock origin and each fragment's bounds, checksum and byte count. Fragment
timestamps and captured bytes form the media timeline; receipt order is not presentation time.

The media node uploads original fragments and durable receipts directly to private S3 storage.
The API imports receipts into PostgreSQL, which owns lifecycle and accepted metadata. Media-node
memory and spool files are temporary buffers. Initial upload concurrency is four per node; spool
limits are 256 MiB per recording and 1 GiB per node. Exhaustion or storage failure stops capture,
warns participants and creates an explicit gap; live publishers continue. Recovery retries use
exponential backoff capped at 30 seconds while recording remains desired.

Start creates a recording; Pause stops capture of media, workspace events and session chat; Resume
continues that recording with a fresh workspace snapshot; Stop seals it. A later Start creates a
new recording. Preserve pause/outage duration in the timeline. A resumed snapshot includes the
current workspace after paused edits, but paused events and chat are excluded. Record chat create,
edit and delete history, workspace updates, viewport/presentation events and archived assets.
Awareness and room chat outside the session are excluded. Pending archive copies pin source bytes
against physical cleanup while allowing logical deletion.

`auto_record` defaults to true at creation and can be edited only while scheduled. Existing live
sessions are not started by migration. Managers, administrators and moderators with target-session
access may control recording using a recording-specific capability; members may not. Capture
startup failure leaves the session live with an honest failure indication. Session end first
persists a terminal capture boundary, then allows up to 30 seconds to seal before media cleanup.
Serialize recording commands with the session's durable write boundary, deduplicate command UUIDs,
and fence stale commands/reports using revisions, capture generations and media boot identities.

The worker finalizes only sealed capture generations through the existing transactional job
outbox, claims, retry and dead-letter mechanism (`recordings.finalize.v1`). It verifies original
segments and publishes a deterministic, versioned replay manifest containing tracks, clocks,
fragments, intervals, gaps, events, snapshots and archived assets. Ready output distinguishes
complete and partial coverage; no usable output fails. PostgreSQL atomically accepts the manifest.
Original fragments are replay assets and remain; only temporary files are cleaned up. Playback,
retention and browser handling of H264/Matroska belong to M9-06 and later work.

## Prototype evidence

The native `mahfel-rtc` fixture uses real mediasoup workers/plain transports and GStreamer
pipelines. On 2026-09-28, a controlled ten-minute, three-track Opus/VP8/H264 fixture passed every
fragment's decode-to-EOS check. It produced 421 readable fragments and 13,089,653 bytes. The
cross-track initial offset spread was 2.542 ms; maximum source/capture duration drift was 5.090 ms,
under the 100 ms acceptance threshold. CPU and RSS include synthetic source encoders and decode
verification. This establishes the controlled prototype, not arbitrary browser/network behavior.

A process-kill fixture preserved three finalized fragments; the measured unsealed window was
2.25 seconds. A pause/resume fixture preserved a 2.25-second gap while its publisher continued.
See the [reproduction and resource evidence](../development/recording-spike.md).

The prototype's encoded queue is bounded to 4 MiB and UDP buffers to 256 KiB per socket. The
production adapter adds the spool/upload policy above, durable S3 receipts, PostgreSQL lifecycle
reconciliation and worker finalization. The native spike fixture remains a separate focused test
of the original-codec pipeline.

## Consequences and limitations

Original tracks avoid composition/encoding cost and preserve interactive replay. They require a
manifest-driven player, and H264/Matroska is not uniformly supported by browsers. No claim of native
browser playback is made here.

Abrupt process loss discards the active unfinalized fragment. Restart on the same filesystem can
recover finalized spool files, but node/ephemeral-volume loss also loses everything not yet uploaded.
A five-second target is not a loss bound when keyframes are absent. Bounded buffering, stale-report
fencing, explicit gaps and partial finalization are required for honest coverage. RTP timestamp
mapping must be validated beyond the controlled shared-clock fixture before claiming synchronization
across arbitrary devices.

## Alternatives considered

- Composite recording requires decode, layout and encode, fixes one view of collaboration, and
  expands the agreed scope. It can be added as a separate export path later.
- Raw RTP dumps preserve packets but require reconstruction, timestamp interpretation and container
  generation before playback. Independently readable original-codec fragments simplify recovery.
- Browser MediaRecorder makes durable session recording depend on one browser's availability and
  mixes that browser's chosen sources. It cannot cover all SFU producers reliably.
- FFmpeg can receive and mux RTP but is not selected. GStreamer provides in-process typed lifecycle,
  RTP/RTCP timing and fragment bus events, and meets the controlled spike without generated export.
