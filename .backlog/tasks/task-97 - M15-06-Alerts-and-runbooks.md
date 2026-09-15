---
id: TASK-97
title: M15-06 Alerts and runbooks
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
labels: []
milestone: m-15
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 105000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Dashboards alone do not notify or prescribe action.

Implementation scope: SLO-derived alerts, inhibition/routing, media overload, queue age, DLQ, pool saturation, scrape failure and linked runbooks.

Dependencies: M15-02–M15-05
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Synthetic faults fire expected alert without storm and runbook leads to verified diagnosis.
<!-- AC:END -->
