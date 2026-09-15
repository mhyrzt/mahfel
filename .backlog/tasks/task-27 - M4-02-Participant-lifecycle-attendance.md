---
id: TASK-27
title: M4-02 Participant lifecycle/attendance
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-4
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 35000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Connections are transient but attendance is durable.

Implementation scope: Join authorization, participant state, reconnect identity, join/leave intervals, attendance aggregation.

Dependencies: M4-01, M3-06
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Reconnect does not duplicate participant; attendance accurately records intervals.
<!-- AC:END -->
