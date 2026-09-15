---
id: TASK-30
title: M4-05 Media node registry
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-4
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 38000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Scheduler needs fresh capacity and lifecycle state.

Implementation scope: Redis heartbeat schema/TTL, node identity/address/capacity/draining, signed/private update path and metrics.

Dependencies: M1-06, M3-09
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Dead node expires; draining state persists for heartbeat lifetime; spoofed update rejected.
<!-- AC:END -->
