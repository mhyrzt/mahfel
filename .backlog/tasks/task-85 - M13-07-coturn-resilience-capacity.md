---
id: TASK-85
title: M13-07 coturn resilience/capacity
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
labels: []
milestone: m-13
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 93000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Relay is bandwidth-heavy and a critical fallback.

Implementation scope: Multiple instances/DNS strategy, credential independence, port/address allocation, bandwidth limits, drain and outage tests.

Dependencies: M3-09
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Forced-relay test survives intended failure model and capacity limits are measurable.
<!-- AC:END -->
