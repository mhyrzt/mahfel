---
id: TASK-6
title: M1-05 WebSocket gateway
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-1
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 14000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Signaling and realtime capabilities need a scalable connection boundary.

Implementation scope: Upgrade route, connection lifecycle, heartbeat, typed envelope/versioning, bounded queues, graceful close, metrics.

Dependencies: M1-03
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Multiple clients connect, exchange a test event, detect disconnect, and expose aggregate metrics.
<!-- AC:END -->
