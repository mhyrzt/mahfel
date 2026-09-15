---
id: TASK-58
title: M9-03 Recording lifecycle
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
labels: []
milestone: m-9
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 66000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Control, media, and worker need one durable state machine.

Implementation scope: Requested/starting/recording/stopping/processing/ready/failed states, idempotent commands and reconciliation.

Dependencies: M9-01, M9-02, M8
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Duplicate start/stop is safe; state recovers/reconciles after API restart.
<!-- AC:END -->
