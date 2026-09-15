---
id: TASK-18
title: M3-02 Router/room lifecycle
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-3
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 26000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Every live room needs isolated mediasoup resources.

Implementation scope: Codec capabilities, router create/lookup/delete, ownership map, idempotent gRPC commands and cleanup.

Dependencies: M3-01, M0-05
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Create/delete is idempotent; leaks are absent in repeated lifecycle test.
<!-- AC:END -->
