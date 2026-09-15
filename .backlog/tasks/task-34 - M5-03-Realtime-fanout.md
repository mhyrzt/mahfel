---
id: TASK-34
title: M5-03 Realtime fanout
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-5
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 42000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Multiple API replicas must deliver the same committed events.

Implementation scope: Transaction-to-event boundary, Redis pub-sub/stream decision, local connection hub, sequence/dedupe and backpressure.

Dependencies: M1-05, M1-06, M5-01
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Clients on two API replicas receive committed message once; slow client is bounded.
<!-- AC:END -->
