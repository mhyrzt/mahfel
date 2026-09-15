---
id: TASK-2
title: M1-01 Migration runner and pools
status: To Do
assignee: []
created_date: '2026-09-08 00:30'
labels: []
milestone: m-1
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 10000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Durable features need controlled schema lifecycle and bounded connections.

Implementation scope: SQLx migration command/startup policy, pool configuration, timeouts, health/metrics, disposable DB test harness.

Dependencies: M0
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Forward migration runs once safely; pool failure is observable; integration test provisions schema.
<!-- AC:END -->
