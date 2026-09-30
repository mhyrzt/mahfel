---
id: TASK-61
title: M9-06 Playback and retention
status: Done
assignee:
  - '@codex'
created_date: '2026-09-08 00:32'
updated_date: '2026-09-30 07:27'
labels: []
milestone: m-9
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 69000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Finished recordings need authorized access and lifecycle management.

Implementation scope: Recording API/UI, presigned playback, duration/status, retention/delete policy and audit.

Dependencies: M9-05, M6-04
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Authorized playback works; unauthorized URL cannot be obtained; retention deletion is tested.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Reuse immutable replay reconstruction for authorized seekable playback. Add member/admin replay descriptors with short-lived archived object URLs, manager-authorized durable deletion and audit, claim fencing, retryable storage cleanup and orphan bundle cleanup. Add playback/delete UI with English/Persian states and focused tests. Verify worker container sandbox and real native output, then run affected component gates and local end-to-end recording checks.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Design note: Playback loads the required snapshot and stored presentation assets, then applies timestamped collaboration and shared-viewport events in order alongside A/V. All participants reconstruct the same room view; downloadable video export is a separate optional capability.

Design clarification: Recording playback/export may present reconstructed canvas, document, slides, and A/V in a purpose-built recording layout independent of the live frontend. Keep that presentation layer separate from event/state reconstruction.

Implemented M9-06 as chosen scope: authorized interactive workspace/presentation/chat replay (no browser A/V fragment decoding) plus durable retention deletion.

Backend: replaced the unwired playback scaffolding with an application use case that authorizes members/admins/blocks, reads the immutable manifest, collects content-addressed pages and archived asset keys, and returns short-lived presigned URLs (PlaybackDescriptor). Added GET /v1/sessions/{id}/recordings/{recording_id}/playback and DELETE /v1/sessions/{id}/recordings/{recording_id}; registered both in the OpenAPI document. Extended RecordingPlaybackStore wiring in AppState.

Frontend: regenerated the REST contract, added features/recordings/recording-player.tsx (reuses the shared replay engine via a new asset-URL resolver, lazy-imported to keep excalidraw/pdfjs out of unrelated bundles), features/recordings/replay/manifest.ts (descriptor -> ReplayInput with precise RFC3339 nanosecond parsing), play/delete actions, EN/FA copy and a Manager story.

Retention: worker five-minute sweep now deletes every object under recordings/{recording}/ before finalizing DB cleanup; finish_recording_cleanup removes bundle and export rows and the export attempts, and an incomplete object delete is retried rather than finalized.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Delivered M9-06 authorized interactive replay and retention. The API exposes a playback descriptor (GET .../recordings/{id}/playback) that authorizes current members or administrators, enforces active blocks, and returns five-minute presigned URLs for the immutable manifest, its content-addressed pages and archived presentation assets; unauthorized callers get 404 with no URL. Managers with session access delete recordings via DELETE .../recordings/{id} (soft-delete, status failed, revision fence, audit, pin release). The worker sweep deletes every object under the recording prefix, then finalizes DB cleanup of bundle and export rows with retry on partial failure. The browser reuses the shared replay engine through a new asset-URL resolver and a manifest loader; the recording list adds play/delete actions with EN/FA copy and a Manager story. Verified with mahfel-application playback unit tests, 150 mahfel-api tests, the PostgreSQL recordings integration (including retention deletion), 149 frontend tests, strict clippy, prettier/ESLint, Next.js production build, Storybook build and mkdocs strict.
<!-- SECTION:FINAL_SUMMARY:END -->
