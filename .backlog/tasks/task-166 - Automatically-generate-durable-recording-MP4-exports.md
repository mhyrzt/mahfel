---
id: TASK-166
title: Automatically generate durable recording MP4 exports
status: Done
assignee:
  - '@codex'
created_date: '2026-09-29 09:41'
updated_date: '2026-10-01 06:37'
labels: []
dependencies:
  - TASK-60
priority: medium
ordinal: 174000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Queue exports after finalization, isolate worker concurrency, fence results, stream artifact storage and authorize status/download/retry.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Duplicate delivery and claim loss cannot publish stale exports; originals remain ready on failure.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Verify export against the current worker image and native fixture. 2. Use the cached matching Chromium because the standard Playwright browser-binary download returned a location-based HTTP 403; this is a tooling acquisition issue, not evidence of an MP4 endpoint failure. 3. Preserve export/original independence and verify claims, retry, authorized download and cleanup against the current stack.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
PostgreSQL integration proves automatic queueing, progress fencing, stale acceptance rejection, failure independence and idempotent retry. RustFS multipart upload and bounded downloads pass; worker runtime, orphan ledger and cleanup implemented.

Fresh PostgreSQL lifecycle/authorization/orphan cleanup and RustFS multipart/lifecycle tests pass. Existing worker RabbitMQ/PostgreSQL/RustFS integration passes while export failure stays independent. The SQL checksum query was aligned to the migration linted sha_256 identifier. Accepted artifacts are excluded from cleanup.

Code validation is complete: strict Clippy, backend unit suites, fresh PostgreSQL lifecycle/authorization/orphan cleanup, RustFS multipart and abort policy, separate worker queue integration, full native 1080p fixtures and Storybook browser checks passed. Standard browser download is currently rejected with location-based HTTP 403; cached matching Chromium is being used for runtime image verification.

2026-09-30 final completion: Final worker image generated an accepted composite MP4, independently of interactive playback. Authenticated download matched stored size (1360613 bytes); ffprobe and GStreamer decode verified H264 1920x1080 and AAC 48kHz stereo, duration 13.040333 seconds. PostgreSQL integration covers duplicate delivery, stale claim rejection, failure independence, retry and orphan cleanup. Packaged Chromium acquisition and runtime are verified locally, superseding the earlier CDN packaging blocker.

2026-10-01: TASK-168 and accepted ADR 0030 supersede the earlier packaged Chromium renderer. Native PNG rendering is now provided by the existing Bun/Next.js web application with a dedicated API key; Rust/Yrs and GStreamer own worker replay/composition. The web/worker images build without browser packaging in the worker, and real MP4 fixture rendering against the production web image passes both-stream EOS verification. September Chromium evidence above is historical, not the current packaging.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Durable claim-fenced automatic MP4 exports are implemented; final packaged worker output and authorized download decode successfully.
<!-- SECTION:FINAL_SUMMARY:END -->
