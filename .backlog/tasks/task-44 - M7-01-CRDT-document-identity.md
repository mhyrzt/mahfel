---
id: TASK-44
title: M7-01 CRDT document identity
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-7
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 52000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Notes/boards need ownership, type, permission, and version boundaries.

Implementation scope: Document metadata/schema, classroom/session association, ACL, lifecycle and protocol version.

Dependencies: M1, M2-07, ADR 0012
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Unauthorized sync rejected; each document type/version resolves deterministically.
<!-- AC:END -->
