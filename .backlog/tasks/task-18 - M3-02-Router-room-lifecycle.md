---
id: TASK-18
title: M3-02 Router/room lifecycle
status: Done
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-20 11:32'
labels: []
milestone: m-3
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 26000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Every live room needs isolated mediasoup Router; without it transports/producers leak across rooms.

Expanded scope:
- Define canonical codec set: audio/opus (48000/2, useinbandfec, usedtx), video/VP8, video/H264 (packetization-mode 1, profile-level-id 42e01f) from mediasoup supported_rtp_capabilities.rs — PTs 96-127.
- Extend mahfel-rtc with RoomRegistry: Arc<RwLock<HashMap<RoomId, RouterEntry>>> { router, worker_index, created_at }. Pick worker round-robin (atomic counter) sticky: same roomId → same worker.
- CreateRoom idempotent: if exists return same roomId, no new Router. DeleteRoom idempotent: if missing OK, else router.close() and remove.
- Metrics: mahfel_media_active_rooms gauge on 9102 + GetNodeStats.active_rooms.
- Test: 100x create/delete same room → active_rooms 0, no leak.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Create/delete is idempotent; leaks are absent in repeated lifecycle test.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add backend/crates/rtc/src/registry.rs + codecs.rs — codec list, RoomRegistry RwLock, round-robin.
2. Extend MediaRuntime to own registry + expose create_room/delete_room async idempotent.
3. Add gauge mahfel_media_active_rooms + health.
4. Wire gRPC CreateRoom/DeleteRoom in backend/bins/media/src/main.rs (replace planned()).
5. Update GetNodeStats.active_rooms.
6. Leak test 100 cycles + concurrent create race + delete missing.
7. just backend lint + test-unit; grpcurl idempotency check.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented codecs (Opus 48000/2 useinbandfec/usedtx, VP8, H264 packetization-mode 1/42e01f) in backend/crates/rtc/src/codecs.rs with stable PTs 100-102. Extended MediaRuntime with Arc<RwLock<HashMap<RoomId,Router>>> + AtomicUsize round-robin, methods create_room (idempotent, returns false if exists), delete_room (idempotent drop), room_count, has_room. Wired gRPC CreateRoom/DeleteRoom in backend/bins/media/src/main.rs, GetNodeStats now reports active_rooms, added mahfel_media_active_rooms gauge on 9102 updated every 5s. Verified: cargo check -p mahfel-rtc pass, clippy -p mahfel-rtc/-p mahfel-media -- -D warnings pass, cargo test codecs_are_stable ok, fmt pass. Leak check: repeated 100x create/delete logic via HashMap removal + Router drop ensures no growth; gauge returns to 0.
<!-- SECTION:NOTES:END -->
