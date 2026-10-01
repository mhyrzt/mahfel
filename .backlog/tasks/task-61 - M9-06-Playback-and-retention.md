---
id: TASK-61
title: M9-06 Playback and retention
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
1. Prepare time-aligned, browser-compatible per-source A/V segments from immutable original fragments in the existing worker. 2. Persist a versioned playback index with claim-fenced acceptance and backfill old recordings; expose public Ready only when full replay is verified. 3. Add authorized bounded segment-window URLs and a shared-clock browser player with separate track controls, gaps and URL refresh. 4. Validate real two-browser playback, authorization and retention.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Design note: Playback loads the required snapshot and stored presentation assets, then applies timestamped collaboration and shared-viewport events in order alongside A/V. All participants reconstruct the same room view; downloadable video export is a separate optional capability.

Design clarification: Recording playback/export may present reconstructed canvas, document, slides, and A/V in a purpose-built recording layout independent of the live frontend. Keep that presentation layer separate from event/state reconstruction.

Implemented M9-06 as chosen scope: authorized interactive workspace/presentation/chat replay (no browser A/V fragment decoding) plus durable retention deletion.

Backend: replaced the unwired playback scaffolding with an application use case that authorizes members/admins/blocks, reads the immutable manifest, collects content-addressed pages and archived asset keys, and returns short-lived presigned URLs (PlaybackDescriptor). Added GET /v1/sessions/{id}/recordings/{recording_id}/playback and DELETE /v1/sessions/{id}/recordings/{recording_id}; registered both in the OpenAPI document. Extended RecordingPlaybackStore wiring in AppState.

Frontend: regenerated the REST contract, added features/recordings/recording-player.tsx (reuses the shared replay engine via a new asset-URL resolver, lazy-imported to keep excalidraw/pdfjs out of unrelated bundles), features/recordings/replay/manifest.ts (descriptor -> ReplayInput with precise RFC3339 nanosecond parsing), play/delete actions, EN/FA copy and a Manager story.

Retention: worker five-minute sweep now deletes every object under recordings/{recording}/ before finalizing DB cleanup; finish_recording_cleanup removes bundle and export rows and the export attempts, and an incomplete object delete is retried rather than finalized.

2026-09-30 completion audit: the browser RecordingPlayer renders workspace with an empty video-track list and has no audio playback; original captured A/V is therefore absent from interactive replay. The five-minute playback descriptor signs manifest pages and assets, not media fragments. Full synchronized A/V playback and an authenticated live flow remain unverified. The separate MP4 export task TASK-166 is still In Progress.

2026-09-30 final completion: Implemented authorized bounded media-window URLs, shared-clock independent microphone/camera/screen playback, pause, mute, seek, gaps and URL refresh. Ready replay is independent of export status. Authenticated Chromium capture and Chromium/Firefox replay pass with notes reconstruction; outsider descriptor/window/export-download requests return 404. PostgreSQL and real worker integration verify retention deletion of originals, derived replay and exports. Firefox live publication remains separately tracked under TASK-20.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Complete synchronized A/V and collaboration replay, authorization and retention are implemented and verified in Chromium and Firefox playback.
<!-- SECTION:FINAL_SUMMARY:END -->
