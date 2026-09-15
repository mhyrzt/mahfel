---
id: TASK-40
title: M6-03 Presigned upload
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-6
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 48000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: APIs should authorize but not proxy file bodies.

Implementation scope: Initiate contract, limits, short expiry, required headers/checksum, completion verification and abandoned cleanup.

Dependencies: M6-01, M6-02
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Browser uploads directly; oversized/wrong checksum cannot become ready metadata.
<!-- AC:END -->
