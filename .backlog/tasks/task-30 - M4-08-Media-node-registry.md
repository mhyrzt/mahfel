---
id: TASK-30
title: M4-08 Media node registry
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-20 19:02'
labels:
  - backend
  - media
milestone: m-4
dependencies:
  - TASK-7
  - TASK-25
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 38000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Give the placement scheduler fresh, trustworthy capacity and lifecycle state. Define a Redis heartbeat schema with TTL, node identity, address, capacity, and draining state, expose a private authenticated update path so a spoofed or unauthenticated update is rejected, and publish bounded registry metrics with explicit namespaced key lifecycle.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A dead node expires after the TTL and is excluded from placement.
- [ ] #2 Draining state persists for the heartbeat lifetime, and the node is removed after expiry.
- [ ] #3 The update path is private and authenticated, and a spoofed or unauthenticated update is rejected.
- [ ] #4 Registry metrics have bounded cardinality and keys are namespaced with explicit lifecycle semantics.
- [ ] #5 Losing the registry degrades placement with an explicit unavailable outcome rather than an incorrect placement.
<!-- AC:END -->
