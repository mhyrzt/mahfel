---
id: TASK-51
title: M8-02 Reliable publisher
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-8
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 59000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: A successful DB request must not silently lose its job.

Implementation scope: Confirm-mode channel, mandatory routing, timeout/error metrics, stable job/idempotency IDs.

Dependencies: M8-01
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Unroutable/nacked publish is surfaced; confirmed publish integration test passes.
<!-- AC:END -->
