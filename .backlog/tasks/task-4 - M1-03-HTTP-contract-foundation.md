---
id: TASK-4
title: M1-03 HTTP contract foundation
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-1
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 12000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Clients need stable errors, pagination, validation, and versions.

Implementation scope: /api/v1, request IDs, JSON error envelope, validation/extractors, pagination convention, CORS/security middleware.

Dependencies: M0, M1-02
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Contract tests cover success/error/request ID; invalid input never becomes generic 500.
<!-- AC:END -->
