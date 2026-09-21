---
id: TASK-27
title: M4-02 Participant lifecycle and reconnect identity
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-21 09:51'
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
- [ ] #7 A second concurrent device is allowed: one participant holds multiple connections, and the participant is present while at least one connection is live.
- [ ] #8 Removing or muting a participant resolves unambiguously across that participant's connections, and no duplicate participant row is created by a second device.
<!-- AC:END -->

## Comments

<!-- COMMENTS:BEGIN -->
author: @mahyar
created: 2026-09-21 09:51
---
Decisions recorded before implementation:
A2 concurrent devices: one participant may hold several live connections; presence is derived from connection count (present while >=1 live), not from a single connection. Roster shows one participant; media de-duplication and per-connection targeting are the responsibility of this task's model, consumed by M4-05.
B1 blocking deferred to M4-05: M4-02 ends the connection only; the durable block list and its join-path check land in M4-05 without reworking the participant model (AC 6).
---
<!-- COMMENTS:END -->
