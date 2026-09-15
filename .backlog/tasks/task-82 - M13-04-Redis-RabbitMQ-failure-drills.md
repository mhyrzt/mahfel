---
id: TASK-82
title: M13-04 Redis/RabbitMQ failure drills
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
labels: []
milestone: m-13
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 90000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Secondary system failures should not cause restart storms or corruption.

Implementation scope: Partition/restart tests, capability degradation matrix, reconnect/backoff, circuit behavior and operator runbooks.

Dependencies: M5, M8
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Each injected outage matches documented readiness/data-loss behavior and recovers automatically.
<!-- AC:END -->
