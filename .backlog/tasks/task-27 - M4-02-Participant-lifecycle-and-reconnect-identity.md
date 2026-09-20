---
id: TASK-27
title: M4-02 Participant lifecycle and reconnect identity
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-20 19:02'
labels:
  - backend
  - api
milestone: m-4
dependencies:
  - TASK-26
  - TASK-15
  - TASK-22
documentation:
  - docs/backlog/roadmap.md
priority: high
ordinal: 35000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
A connection is transient but a principal presence in a live session is a stable identity. Introduce a durable per-session Participant keyed by principal (user or service principal) that survives reconnect, replacing the LMS teacher/student attendance notion. Cover join authorization, explicit participant state, reconnect identity with a connection generation, an explicit multi-device policy, and leave/removal cleanup.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Reconnecting with a new transport or WebSocket reuses the same participant identity, and no duplicate participant is created.
- [ ] #2 Join is authorized deny-by-default against room membership, visibility, and admission policy.
- [ ] #3 Participant state transitions are explicit and audited, and leave or removal is terminal for that connection but not for an authorized rejoin unless the principal is blocked.
- [ ] #4 The multi-device policy is explicit and documented, and behavior for a second concurrent device is tested.
- [ ] #5 WebSocket join and leave events carry the stable participant identity used by the roster.
<!-- AC:END -->
