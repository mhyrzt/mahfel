---
id: TASK-27
title: M4-02 Participant lifecycle and reconnect identity
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-21 09:45'
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
A connection is transient but an actor's presence in a live session is a stable identity. Introduce a durable per-session Participant keyed by actor (user or service principal) that survives reconnect, replacing the LMS teacher/student attendance notion. Cover join authorization, explicit participant state, reconnect identity with a connection generation, an explicit multi-device policy, and leave/removal cleanup. Admission policy and blocking are owned by M4-06 and M4-05 and extend this task rather than being decided here.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Reconnecting with a new transport or WebSocket reuses the same participant identity, and no duplicate participant is created.
- [ ] #2 Join is authorized deny-by-default against room membership and visibility; admission policy is applied by M4-06 through the same join path.
- [ ] #3 Participant state transitions are explicit and audited, and leaving or being removed is terminal for that connection while an authorized actor can rejoin.
- [ ] #4 The multi-device policy is explicit, documented, and tested.
- [ ] #5 WebSocket join and leave events carry the stable participant identity used by the roster.
- [ ] #6 The participant identity and join path are reusable by M4-05 blocking and M4-06 admission without reworking the participant model.
<!-- AC:END -->
