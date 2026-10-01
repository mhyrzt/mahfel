---
id: TASK-83
title: M13-05 Media drain and rolling deploy
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
updated_date: '2026-09-30 13:19'
labels: []
milestone: m-13
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 91000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Live rooms cannot be moved like stateless HTTP requests.

Implementation scope: Draining registry state, scheduler exclusion, deadline, termination grace, optional room warning and rollout runbook.

Dependencies: M4-05, M4-06
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 New rooms avoid drain; existing rooms finish within policy; node exits gracefully.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
M9 runtime audit (2026-09-30): local media container recreation left Redis hash mahfel:media_nodes:record:media-local with draining=1 even while the heartbeat payload reported draining=false. HEARTBEAT_SCRIPT preserves old draining=1 indefinitely across a quick restart with the same node ID. New placements returned 503 until the local ephemeral flag was reset. Cover restart identity/drain reset semantics in this resilience task; M9 playback verification used the healthy node after clearing the stale local flag.
<!-- SECTION:NOTES:END -->
