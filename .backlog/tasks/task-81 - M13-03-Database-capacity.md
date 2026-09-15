---
id: TASK-81
title: M13-03 Database capacity
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
labels: []
milestone: m-13
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 89000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Replica growth can exhaust PostgreSQL or amplify slow queries.

Implementation scope: Global pool budget, PgBouncer decision, statement/lock timeouts, query metrics, index review and overload behavior.

Dependencies: M1, workload profiles
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Saturation test sheds/degrades predictably; dashboards expose pool waits and slow operations.
<!-- AC:END -->
