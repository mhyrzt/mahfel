---
id: TASK-65
title: M10-04 Speaker mapping
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
labels: []
milestone: m-10
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 73000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Media track identity and human speakers can diverge across reconnects.

Implementation scope: Track/participant intervals, diarization reconciliation, unknown speaker handling and correction API.

Dependencies: M4-02, M10-03
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Reconnect fixture maps segments correctly; ambiguous speaker remains explicitly unknown.
<!-- AC:END -->
