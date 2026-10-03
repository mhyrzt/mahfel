---
id: TASK-104
title: M16-06 Media placement/draining rollout
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
updated_date: '2026-10-03 11:28'
labels: []
milestone: m-17
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 112000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Scheduler identity must match platform replica/network lifecycle.

Implementation scope: Registry address publication, node resources, probes, drain hook/grace period, autoscaling constraints and rollback.

Dependencies: M16-03, M4-06, M13-05
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 New rooms distribute across two nodes; rolling removal drains without new placement.
<!-- AC:END -->
