---
id: TASK-60
title: M9-05 Finalization worker
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
ordinal: 68000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Finalize sealed immutable original-codec fragments and collaboration artifacts through M8 outbox, claims, retries and DLQ. Verify checksums, containers and clocks; derive browser-compatible per-source media and atomically accept deterministic replay manifests and playback indexes. Composite MP4 export is TASK-166; playback UI and retention are TASK-61.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Finalization retry is safe; original segments decode, checksums and synchronized track clocks are verified, and a fenced replay manifest preserves intervals, gaps, events, snapshots and archived assets.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Verify immutable originals and archive assets through real storage boundaries. 2. Derive bounded per-source browser media and content-addressed replay index under existing worker claims. 3. Accept Ready atomically, backfill legacy recordings without replacing original manifests, and deduplicate export queueing. 4. Validate native codecs and real PostgreSQL/RustFS/RabbitMQ retry, fencing and cleanup.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Completed recordings.finalize.v1 through the existing M8 outbox, quorum queue/DLQ, renewable claims, bounded retries and operator replay. Finalization imports durable receipts, verifies SHA-256/size/container decoding and archived assets, and writes deterministic content-addressed replay pages/manifest. Tracks, source clocks, session-relative event/interval bounds, pause/media gaps, uncertain coverage, snapshots, event histories and asset metadata are retained. Acceptance is atomic under the current lease/claim and sealed capture generation. Corrupt fragments produce partial ready output when usable data remains; empty unusable recordings fail. Duplicate attempts reuse accepted results; dead-job replay restores finalizing for the same sealed generation.

Validation: real PostgreSQL/RustFS/RabbitMQ test proves storage-failure retry, corrupt-fragment partial coverage, original-byte preservation, archive-copy verification/pin release, duplicate finalization and empty-recording failure. PostgreSQL tests prove stale claim rejection and operator replay. Native Opus/VP8/H264 readability and clock fixtures, strict workspace clippy/build, SQL, protobuf and docs pass.

Earlier composition/export notes are superseded: FFmpeg, video reconstruction, transcoding, composition and MP4/video export are excluded. Playback, retention and orphan manifest cleanup remain TASK-61. No images were built or deployment/browser recording session tested.

2026-09-30 completion audit: a fresh MAHFEL_TESTCONTAINERS=1 mahfel-worker integration run failed while finalizing a referenced asset: job retry reason asset_copy_unavailable, then timed out waiting for recording finalization. The prior successful result is not sufficient to call this fresh run green. Source of the copy failure still needs diagnosis; no retry/time limit workaround was applied.

2026-09-30 final completion: Approved completion scope preserves immutable original fragments and derives bounded browser-compatible H264/AAC segments plus a content-addressed index in the existing worker. Ready acceptance is claim-fenced and capped at 8MiB/10000 segments. Legacy backfill preserves original manifest pointers and deduplicates exports. Real PostgreSQL/RabbitMQ/RustFS worker integration passes, including asset archive verification, retry, original preservation, browser media and retention. Native Opus/VP8/H264 checks pass; final packaged worker runs successfully. This supersedes earlier unresolved asset-copy and packaging notes.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Finalization verifies immutable originals, produces accepted synchronized browser media and safely backfills legacy recordings; real infrastructure and native fixtures pass.
<!-- SECTION:FINAL_SUMMARY:END -->
