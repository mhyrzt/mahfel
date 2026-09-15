---
id: TASK-53
title: M8-04 Retry and DLQ
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-8
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 61000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Infinite redelivery hides poison jobs and creates load.

Implementation scope: Bounded delayed retries, attempt metadata, exponential policy, DLX/DLQ, operator inspection/replay procedure.

Dependencies: M8-03
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Transient fixture retries then succeeds; poison fixture reaches DLQ after exact limit.
<!-- AC:END -->
