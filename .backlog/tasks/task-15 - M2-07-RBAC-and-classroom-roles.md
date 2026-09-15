---
id: TASK-15
title: M2-07 RBAC and classroom roles
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-2
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 23000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Authentication alone does not enforce classroom actions.

Implementation scope: Global admin plus teacher/student/moderator role model, policy checks, audit context, role assignment constraints.

Dependencies: M1-07, M2-03
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Route/service tests prove deny-by-default and role-specific allow/deny matrix.
<!-- AC:END -->
