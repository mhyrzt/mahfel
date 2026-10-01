---
id: TASK-59
title: M9-04 Capture and upload
status: Done
assignee:
  - '@codex'
created_date: '2026-09-08 00:32'
updated_date: '2026-09-30 15:19'
labels: []
milestone: m-9
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 67000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Media fragments must reach durable storage without relying on container disk.

Implementation scope: Media tap, bounded spool, segment naming/checksums, multipart/direct S3 upload, backpressure and failure metrics.

Dependencies: M9-01, M9-03, M6-02
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Captured segments appear in S3; storage slowdown is bounded and produces explicit failure.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Use measured original encoded mediasoup/GStreamer capture. Target 5-second segments; bounded 4 uploads/node, 256MiB recording spool and 1GiB node spool. Direct private S3 with deterministic keys/checksums and durable receipts. Transactionally record collaboration/session-chat event history and baseline snapshots; exclude awareness/paused events. Archive referenced bytes with deletion-safe pins. No transcoding.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Integrated original encoded Opus/VP8 WebM and H264 Matroska capture through loopback mediasoup plain transports and GStreamer, including RTCP feedback and selected simulcast spatial layer. Dynamic producer taps retain actor/source identities and shared clock origins. Five-second target fragmentation, four uploads per node, recording/node spool thresholds, atomic journals, direct private S3 fragments/receipts and bounded-cardinality failure metrics are implemented. Prior-generation pending files count against the recording quota. Upload failure preserves journals and restart retries them; exhaustion stops capture and recovery resumes it without stopping publishers.

Collaboration/session-chat history, current-workspace snapshots and referenced asset copies/pins are durable. Paused history and awareness are excluded. Archive copying runs independently of media reconciliation; pending pins prevent physical metadata cleanup until bytes are verified.

Validation: native codec decode/pause/crash fixtures, coordinator fencing/quota/restart tests, real S3 fragment/archive integration and PostgreSQL event/pin tests pass. Spool thresholds allow bounded in-flight sealing headroom; asynchronous media tails must be clipped by replay intervals, documented in docs/deployment/recordings.md. Container images/deployment were not exercised.

2026-09-30 final completion: final media image exercised by authenticated Chromium microphone/camera/screen capture. Shared receive-clock fix removes timestamping failures; native Opus/VP8/H264 clock checks pass. Derived tracks play in Chromium and Firefox. These results supersede previous notes that images or browser capture were not exercised.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Completed native original-track capture, private S3 uploads with recoverable journals, spool pressure handling, collaboration/chat capture and deletion-safe asset archives. Native and real storage/database failure-path tests pass.
<!-- SECTION:FINAL_SUMMARY:END -->
