---
id: TASK-54
title: M8-05 Idempotency
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-8
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 62000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: AMQP is at-least-once, so duplicate execution is normal.

Implementation scope: Handler idempotency key/store, atomic claim/result, stale lease and per-job guidance.

Dependencies: M8-03, M1-01
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Duplicate delivery cannot duplicate durable effect; crashed claim recovers safely.
<!-- AC:END -->
