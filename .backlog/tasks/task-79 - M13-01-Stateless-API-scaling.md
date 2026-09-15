---
id: TASK-79
title: M13-01 Stateless API scaling
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
labels: []
milestone: m-13
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 87000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Hidden in-memory ownership breaks multi-replica behavior.

Implementation scope: Audit sessions, WebSockets, collaboration, signaling, fanout and idempotency; externalize/reconstruct required state.

Dependencies: M1–M12
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Adding API replica needs no migration/sticky HTTP assumption and cross-replica tests pass.
<!-- AC:END -->
