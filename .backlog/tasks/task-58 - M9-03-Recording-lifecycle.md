---
id: TASK-58
title: M9-03 Recording lifecycle
status: Done
assignee:
  - '@codex'
created_date: '2026-09-08 00:32'
updated_date: '2026-09-29 08:51'
labels: []
milestone: m-9
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 66000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Persist starting, recording, paused, stopping, finalizing, ready and failed states with idempotent controls, durable capture generations and restart reconciliation. Dependencies: M9-01, M9-02 and M8.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Duplicate start/stop is safe; state recovers/reconciles after API restart.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Persist capture intent, idempotent command UUIDs, capture epochs/revisions, intervals/gaps and reconciliation. Pause omits all capture; Resume rebaselines current workspace; Stop finalizes and a later Start creates another recording. Failures warn and recover without ending sessions. Fence stale media reports and seal before session media cleanup.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented PostgreSQL-authoritative lifecycle, per-session serialization, UUID command replay, automatic live-session capture intent and terminal stop intent. API reconciliation uses persisted intent after restart, bounded parallel recording groups and private media RPCs. Generations, revisions and media boot IDs fence stale reports; media restart creates a fresh generation and explicit partial coverage. Pause excludes subsequent workspace/chat events; Resume rebaselines current workspace under the same recording. Stop enqueues finalization only after sealing; later Start creates another recording. Session end preserves the terminal boundary and waits at most 30 seconds before cleanup/explicit loss.

Validation: real PostgreSQL proves command deduplication/audits, paused-event exclusion, resume snapshots, boot/revision fencing, sealed-job deduplication, stale-claim rejection and dead-job replay. Native coordinator tests prove sealed generations cannot reactivate and recovery after spool pressure/upload failure. API/domain/application tests and strict workspace lint/build pass. Runtime behavior and asynchronous media-boundary clipping are documented in docs/deployment/recordings.md.

Structural follow-up requested by user: grouped PostgreSQL recording store/capture/finalization under crates/postgres/src/recording/ and application operations/artifacts/finalization under crates/application/src/recordings/. Module roots contain declarations and re-exports; removed the special finalize path attribute. Existing public imports are preserved, including the recording_artifacts module re-export. Behavior and SQL are unchanged. Validation: cargo fmt check, strict workspace/all-targets/all-features Clippy, and 121 application/PostgreSQL library tests pass. Infrastructure integration tests were not rerun for this file/module-only move.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Completed durable lifecycle and restart reconciliation with idempotent controls, generation/boot fencing, pause/resume baselines and bounded session-end sealing. Verified against PostgreSQL, native media and claim/replay regression tests.
<!-- SECTION:FINAL_SUMMARY:END -->
