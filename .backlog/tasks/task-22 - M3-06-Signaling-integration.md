---
id: TASK-22
title: M3-06 Signaling integration
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-3
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 30000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Browser must never call private media gRPC directly.

Implementation scope: API WebSocket signaling commands, authorization, correlation/timeouts, media client retries/status mapping.

Dependencies: M1-05, M2-07, M3-03–M3-05
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 End-to-end signaling works; gRPC remains private; media bytes bypass API.
<!-- AC:END -->
