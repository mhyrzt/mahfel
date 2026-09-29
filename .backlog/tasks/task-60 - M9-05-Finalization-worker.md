---
id: TASK-60
title: M9-05 Finalization worker
status: Done
assignee:
  - '@codex'
created_date: '2026-09-08 00:32'
updated_date: '2026-09-28 14:48'
labels: []
milestone: m-9
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 68000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Finalize sealed original-codec fragments and collaboration replay artifacts through the M8 outbox, claims, retries and DLQ. Verify checksums/container readability and atomically accept a deterministic versioned replay manifest with complete or partial coverage. FFmpeg, reconstruction, transcoding, composition, MP4/video export, playback UI and retention are excluded.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Finalization retry is safe; original segments decode, checksums and synchronized track clocks are verified, and a fenced replay manifest preserves intervals, gaps, events, snapshots and archived assets.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Finalize sealed recording generations via recordings.finalize.v1 and existing M8 outbox/retry/claims/DLQ machinery. Verify original segment checksums/readability, preserve track timing and gaps, assemble versioned replay manifest/events/snapshots/assets, publish ready complete/partial atomically and safely under retries. No FFmpeg, video reconstruction, transcoding, composition or MP4/video exports. Leave playback and retention to TASK-61.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Completed recordings.finalize.v1 through the existing M8 outbox, quorum queue/DLQ, renewable claims, bounded retries and operator replay. Finalization imports durable receipts, verifies SHA-256/size/container decoding and archived assets, and writes deterministic content-addressed replay pages/manifest. Tracks, source clocks, session-relative event/interval bounds, pause/media gaps, uncertain coverage, snapshots, event histories and asset metadata are retained. Acceptance is atomic under the current lease/claim and sealed capture generation. Corrupt fragments produce partial ready output when usable data remains; empty unusable recordings fail. Duplicate attempts reuse accepted results; dead-job replay restores finalizing for the same sealed generation.

Validation: real PostgreSQL/RustFS/RabbitMQ test proves storage-failure retry, corrupt-fragment partial coverage, original-byte preservation, archive-copy verification/pin release, duplicate finalization and empty-recording failure. PostgreSQL tests prove stale claim rejection and operator replay. Native Opus/VP8/H264 readability and clock fixtures, strict workspace clippy/build, SQL, protobuf and docs pass.

Earlier composition/export notes are superseded: FFmpeg, video reconstruction, transcoding, composition and MP4/video export are excluded. Playback, retention and orphan manifest cleanup remain TASK-61. No images were built or deployment/browser recording session tested.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Finalized sealed original-track replay bundles with checksum/decode verification, explicit partial/unusable outcomes and claim-fenced manifests. Real infrastructure tests cover retry, corruption, archived assets, duplicates and replay; video export and M9-06 remain outside scope.
<!-- SECTION:FINAL_SUMMARY:END -->
