---
id: TASK-27
title: M4-02 Participant lifecycle and reconnect identity
status: In Progress
assignee:
  - '@mahyar'
created_date: '2026-09-08 00:31'
updated_date: '2026-09-22 20:06'
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
- [x] #1 Reconnecting with a new transport or WebSocket reuses the same participant identity, and no duplicate participant is created.
- [x] #2 Join is authorized deny-by-default against room membership and visibility; admission policy is applied by M4-06 through the same join path.
- [x] #3 Participant state transitions are explicit and audited, and leaving or being removed is terminal for that connection while an authorized actor can rejoin.
- [x] #4 The multi-device policy is explicit, documented, and tested.
- [x] #5 WebSocket join and leave events carry the stable participant identity used by the roster.
- [x] #6 The participant identity and join path are reusable by M4-05 blocking and M4-06 admission without reworking the participant model.
- [x] #7 A second concurrent device is allowed: one participant holds multiple connections, and the participant is present while at least one connection is live.
- [x] #8 Removing or muting a participant resolves unambiguously across that participant's connections, and no duplicate participant row is created by a second device.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Domain (`mahfel-domain/src/participant.rs`): `entity_id!(ParticipantId)`, `ParticipantStatus { Active, Left, Removed }` with stable storage names, `Participant` aggregate with state-checked `leave`/`remove` (terminal) and `join`. Unit tests for storage names and transitions.
2. Migration `0006_participant.sql`: `mahfel.session_participants` with partial unique index `participants_one_active_per_session_actor_idx` on `(session_id, actor_id) WHERE status = 'active'` so reconnect or a second device cannot duplicate a row. `actor_id` references `mahfel.actors`.
3. Port + adapter: `ParticipantStore` (`get`, `get_active`, `insert`, `save`, `list_for_session`, `count_active`) returning `BoxFuture` per ADR 0023; `PostgresParticipantStore`; unique violation maps to `StoreError::Conflict`.
4. Use cases `crates/application/src/sessions/{join,leave}.rs`: session must be live, caller is manager or member of the room, deny-by-default; reconnect returns the existing active participant rather than inserting; leave is terminal for that connection.
5. WebSocket join path: `join`/`leave` client frames, `joined`/`left`/participant roster server frames; registry gains an actor-to-connections association plus a per-connection generation so presence is '>=1 live connection' (AC 7) and removal resolves across all connections (AC 8).
6. Reconnect generation is connection-scoped (in-process), not a durable column: durable identity is one row per `(session, actor)` and must not mutate on reconnect. Guard is therefore replica-local; cross-replica ordering is deferred to M1-06 (Redis pub/sub, ADR 0017). Documented on the type and in the architecture docs.
7. Tests: domain transitions; reconnect over two sockets yields one participant id; second device does not duplicate the row; unauthorized join rejected; leave terminal then rejoinable.
8. Docs: multi-device policy and the generation limitations in the nearest authoritative document.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
AC 6 closed by TASK-125 (M4-06). Admission layered onto the existing join path without reshaping the participant model: join_session gained a JoinOutcome return and a waiting branch, and Participant, ParticipantStore, and the (session, actor) uniqueness invariant were all unchanged. The lobby reuses the same participant identity on approval, including reusing an existing active participant, so the model proved reusable as claimed. AC 7 (multi-device presence counting) and AC 8 (cross-connection targeting) remain open and belong to TASK-28, where moderation actually needs to resolve a participant across its connections.

Acceptance criteria 7 and 8 are closed by TASK-28 (M4-05).

AC 7, multi-device presence: ConnectionRegistry::connection_count_for_participant counts a participant's live connections, and handle_leave consults it after detaching the leaving connection. A participant is present while at least one connection is live, and ParticipantLeft is emitted only when the last connection goes. Previously handle_leave emitted it unconditionally, so closing one of two devices told the roster the participant had left while the other device was still publishing. Covered by six registry tests, including one participant across two connections and an authenticated connection that has not joined.

AC 8, cross-connection resolution: moderation keys on the actor rather than a connection, so a mute or removal resolves across every device a participant holds: producers are paused or closed once for the principal, and the participant is ended once. Fan-out uses broadcast_to_session, which reaches the target as well as the roster, because the target must learn it was muted or removed. The second device does not duplicate the participant row, which the partial unique index on (session_id, actor_id) WHERE status = 'active' already guaranteed from this task.

Replica caveat, unchanged: the registry is replica-local, so a participant with devices attached to two replicas is counted once per replica. Cross-replica presence and targeting need the connection hub (M4-05 depends on TASK-34, M5-03, ADR 0017), which is why these were testable here single-replica only.
<!-- SECTION:NOTES:END -->

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
