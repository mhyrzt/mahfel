---
id: TASK-166
title: Automatically generate durable recording MP4 exports
status: In Progress
assignee:
  - '@codex'
created_date: '2026-09-29 09:41'
updated_date: '2026-09-29 13:19'
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
- [ ] #1 Duplicate delivery and claim loss cannot publish stale exports; originals remain ready on failure.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Atomically enqueue exports after usable original finalization; persist independent export lifecycle and outbox job; fence render/upload acceptance and retries by current manifest/revision/claim; isolate export intake from ordinary jobs; authorize member/admin downloads and manager retries; clean abandoned uploads/scratch.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
PostgreSQL integration proves automatic queueing, progress fencing, stale acceptance rejection, failure independence and idempotent retry. RustFS multipart upload and bounded downloads pass; worker runtime, orphan ledger and cleanup implemented.

Fresh PostgreSQL lifecycle/authorization/orphan cleanup and RustFS multipart/lifecycle tests pass. Existing worker RabbitMQ/PostgreSQL/RustFS integration passes while export failure stays independent. The SQL checksum query was aligned to the migration linted sha_256 identifier. Accepted artifacts are excluded from cleanup.

Code validation is complete: strict Clippy, backend unit suites, fresh PostgreSQL lifecycle/authorization/orphan cleanup, RustFS multipart and abort policy, separate worker queue integration, full native 1080p fixtures and Storybook browser checks passed. Standard browser download is currently rejected with location-based HTTP 403; cached matching Chromium is being used for runtime image verification.
<!-- SECTION:NOTES:END -->
