---
id: TASK-38
title: M6-01 Object metadata and ACL
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-6
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 46000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Artifact bytes need authoritative ownership/permission metadata.

Implementation scope: Object key strategy, owner/classroom/session scope, status, size/MIME/checksum, ACL service and migrations.

Dependencies: M1, M2-07
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Metadata constraints/ACL matrix tests pass; keys are collision-resistant and non-guessable.
<!-- AC:END -->
