---
id: TASK-33
title: M5-02 History and mutations
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-5
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 41000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Large histories need stable paging and controlled edits/deletes.

Implementation scope: Cursor pagination, edit window/policy, soft delete, revision/audit data and API contracts.

Dependencies: M5-01
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Pagination has no gaps/duplicates under inserts; mutation permissions tested.
<!-- AC:END -->
