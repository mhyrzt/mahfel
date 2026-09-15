---
id: TASK-29
title: M4-04 Active speaker
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-4
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 37000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: UI layout needs a stable aggregate speaker signal.

Implementation scope: mediasoup observer, smoothing/rate limits, authorized WebSocket event, metric without participant labels.

Dependencies: M3-05, M4-02
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Multiple speakers update UI without event flood; disconnect cleanup works.
<!-- AC:END -->
