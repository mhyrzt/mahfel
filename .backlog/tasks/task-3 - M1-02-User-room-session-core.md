---
id: TASK-3
title: M1-02 User/room/session core
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-1
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 11000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Routes need durable identifiers and invariants without a complex schema.

Implementation scope: Minimal users, rooms, classroom sessions, timestamps/statuses, repositories and migrations.

Dependencies: M1-01
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Repository tests create/read each model and data survives process restart.
<!-- AC:END -->
