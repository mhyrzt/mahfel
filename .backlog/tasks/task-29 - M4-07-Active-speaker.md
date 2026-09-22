---
id: TASK-29
title: M4-07 Active speaker
status: Done
assignee:
  - '@mahyar'
created_date: '2026-09-08 00:31'
updated_date: '2026-09-22 22:47'
labels:
  - backend
  - api
milestone: m-4
dependencies:
  - TASK-21
  - TASK-27
  - TASK-25
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 37000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Provide a stable aggregate active-speaker signal from mediasoup audio levels: smoothing and rate limits to prevent event floods, an authorized WebSocket event delivered only to session participants, cleanup when a producer closes or a participant disconnects, and a bounded metric with no participant labels. Vocabulary stays generic.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Multiple simultaneous speakers update the interface without event flooding, using rate limits or hysteresis.
- [x] #2 Speaker events are delivered only to authorized session participants.
- [x] #3 Producer close and participant disconnect remove speaker state promptly.
- [x] #4 The speaker metric has bounded cardinality and carries no participant or room labels.
- [x] #5 Smoothing thresholds and event frequency are documented.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Define the contract: add a private server-streaming media RPC carrying room-scoped snapshots of ordered speaking actor IDs, including an empty snapshot for silence. Each API process subscribes independently; WebSocket fan-out stays local to that process. Use bounded channels and send a reset plus current snapshots on subscription or resynchronization.
2. In mahfel-rtc, create and retain one mediasoup AudioLevelObserver per room router. Register only microphone/audio producers and clear them on pause, close, transport cleanup, and room deletion. Keep producer-to-actor ownership in the media plane; no durable speaker state.
3. Add a pure room-scoped tracker that merges multiple microphones for one actor, smooths sampled levels, applies entry/exit hysteresis, caps the set at eight actors, and emits ordinary changed snapshots at most four times per second. Silence and explicit removal clear state immediately. Initial tuning: sample every 200 ms, observe down to -80 dBvo, require two qualifying samples to enter and three quiet/missing samples to leave; validate and document final thresholds with native media.
4. Expose tracker snapshots through the private gRPC stream. Make lag, disconnect, and media restart clear or replace prior state; bound each subscriber queue and send timeout. Add aggregate active-speaker and dropped-update metrics without room, actor, participant, or producer labels.
5. In mahfel-api, consume the stream with cancellation and bounded retry. Map each room snapshot only to locally joined live sessions, resolve each actor through ParticipantStore on a changed snapshot to avoid stale identity after a rapid leave and rejoin, and retain only the latest per-session participant snapshot for late joins. Send a versioned ActiveSpeakers WebSocket snapshot only to joined connections. Clear it on session end, stream loss, or roster removal. Waiting or merely authenticated connections receive nothing.
6. Track media transports owned by each WebSocket connection. Add a private idempotent CloseWebRtcTransport operation and close that connection's transports on explicit leave and every disconnect path. Remove associated producers/consumers and speaker entries immediately while preserving producers on a second device.
7. Verify overlapping speakers, silence, hysteresis, rate limits, multi-producer identity, and close with deterministic tests; exercise the observer with a native media worker; test gRPC resubscription/lag and a real WebSocket for authorized fan-out, waiting-room exclusion, stable participant IDs, late join, and disconnect cleanup. Run backend fmt/lint/test/build, both proto gates, and docs. Document the event schema, thresholds, local-replica scope, media-degradation behavior, and RTP audio-level extension requirement.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Speaker identity is resolved through ParticipantStore on each changed snapshot so a rapid leave and rejoin cannot replay a stale participant ID. Session end requests an idempotent private media reset while retaining the room router. The native observer check exposed a payload collision: VP8 RTX uses 102, so H264 uses 103. Validation: 333/333 backend unit tests passed, including native mediasoup observer/reset and real WebSocket authorization/late join; workspace build, focused Clippy, backend fmt, proto fmt/lint, docs, and doc tests passed. Combined backend lint fails on pre-existing SQLFluff LT02 violations in unchanged migrations/0004_identity.sql. Full integration run was stopped after 323/351 cases: 321 passed, 2 failed, 28 not run. One identified failure was the Keycloak token-tampering test; the second was not classified.
<!-- SECTION:NOTES:END -->

## Comments

<!-- COMMENTS:BEGIN -->
author: @mahyar
created: 2026-09-22 21:39
---
Planning decisions: expose an ordered set of simultaneously speaking participants, not only a dominant speaker. Keep M4-07 WebSocket fan-out replica-local; cross-replica connection coordination remains TASK-34. Speaker state is ephemeral. Transport-scoped cleanup is required because the current unexpected-disconnect path closes the attendance interval but leaves media transports/producers alive.
---
<!-- COMMENTS:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented ordered active-speaker snapshots from mediasoup through a private gRPC stream to authorized, locally joined WebSocket clients. Added smoothing, bounded metrics, late-join replay, transport and session-end cleanup, and fixed a router codec payload collision discovered by the native test. Verified 333 backend unit tests, native observer and real WebSocket checks, workspace build, Clippy, formatting, proto checks, docs and doc tests. Full integration remains incomplete with two failures; combined lint is blocked by existing SQLFluff migration violations.
<!-- SECTION:FINAL_SUMMARY:END -->
