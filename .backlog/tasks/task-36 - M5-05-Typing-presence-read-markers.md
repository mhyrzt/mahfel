---
id: TASK-36
title: M5-05 Typing/presence/read markers
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-5
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 44000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Ephemeral UX and durable reading state need separate storage semantics.

Implementation scope: TTL typing/presence in Redis, aggregate events, durable per-user channel cursor, privacy/scale limits.

Dependencies: M1-06, M5-03
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Ephemeral indicators expire; read marker survives restart; no user IDs appear in metrics.
<!-- AC:END -->
