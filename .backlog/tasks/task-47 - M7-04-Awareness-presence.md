---
id: TASK-47
title: M7-04 Awareness/presence
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
ordinal: 55000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Cursors/selections are useful but ephemeral and potentially high-volume.

Implementation scope: Yjs awareness relay, TTL cleanup, privacy fields, throttling and no durable persistence.

Dependencies: M7-02, M1-06
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Disconnect/TTL removes awareness and bursts stay within bounded queues.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Relay bounded Yjs awareness through Redis and local channels; let client TTL and disconnect removal clear ephemeral state.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implementation present. Validation: backend Clippy and formatting, PostgreSQL migration integration, API unit suite, frontend typecheck, lint, formatting and 19 tests, production webpack build. Acceptance needs live multi-client/restart verification before marking done.
<!-- SECTION:NOTES:END -->
