---
id: TASK-31
title: M4-09 Session placement scheduler
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-20 19:02'
labels:
  - backend
  - media
milestone: m-4
dependencies:
  - TASK-30
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 39000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Every participant in a session must reach the same capable media node. Implement atomic placement leases, a weighted capacity policy, sticky lookup for the lifetime of a session, explicit race and failure behavior, and a durable diagnostic record of placement decisions.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Concurrent joins to the same room select exactly one node through an atomic lease.
- [ ] #2 Overloaded, draining, and dead nodes are excluded, and the weighted policy spreads load across healthy nodes.
- [ ] #3 Sticky lookup returns the same node for the session lifetime, and a placement failure returns an explicit unavailable result with safe retry.
- [ ] #4 Redis race tests prove that concurrent lease acquisition cannot double-place a session.
- [ ] #5 Placement decisions (node, room, capacity, reason) are recorded durably and queryable by operators.
<!-- AC:END -->
