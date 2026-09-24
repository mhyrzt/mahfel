---
id: TASK-46
title: M7-03 Persistence/checkpoints
status: In Progress
assignee:
  - '@codex'
created_date: '2026-09-08 00:31'
updated_date: '2026-09-24 06:20'
labels: []
milestone: m-7
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 54000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: In-memory CRDT state must survive restarts without unbounded replay.

Implementation scope: Update log versus snapshot design, atomic checkpoint, compaction, load/recovery and corruption handling.

Dependencies: M7-02
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 State restores after restart; compaction preserves state vector; recovery test covers interrupted checkpoint.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Append document updates to an ordered journal, store checksummed checkpoints, retain historical updates for replay, and recover by replaying the journal after an invalid checkpoint.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Design note: Preserve ordered collaboration changes and checkpoints needed for recording replay, including shared viewport changes. Reconstruct from a snapshot plus subsequent events; ephemeral awareness remains separate from durable document and viewport state.

Implementation present. Validation: backend Clippy and formatting, PostgreSQL migration integration, API unit suite, frontend typecheck, lint, formatting and 19 tests, production webpack build. Acceptance needs live multi-client/restart verification before marking done.
<!-- SECTION:NOTES:END -->
