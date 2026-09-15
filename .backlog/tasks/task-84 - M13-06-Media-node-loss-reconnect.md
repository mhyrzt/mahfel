---
id: TASK-84
title: M13-06 Media node loss/reconnect
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
labels: []
milestone: m-13
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 92000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Unexpected death drops volatile SFU state.

Implementation scope: Detection time, client notification/rejoin, new placement rules, session continuity limits and chaos test.

Dependencies: M3-08, M4-06
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Node-kill scenario reaches documented reconnect outcome without stale placement.
<!-- AC:END -->
