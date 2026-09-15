---
id: TASK-31
title: M4-06 Room placement scheduler
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-4
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 39000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Every participant in a room must reach the same capable node.

Implementation scope: Atomic placement lease, weighted capacity policy, sticky lookup, race/failure behavior, durable diagnostic record.

Dependencies: M4-05
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Concurrent joins select one node; overload/draining excludes node; Redis race tests pass.
<!-- AC:END -->
