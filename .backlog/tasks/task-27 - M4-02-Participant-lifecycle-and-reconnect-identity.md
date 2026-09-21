---
id: TASK-27
title: M4-02 Participant lifecycle and reconnect identity
status: In Progress
assignee:
  - '@mahyar'
created_date: '2026-09-08 00:31'
updated_date: '2026-09-21 12:16'
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
- [ ] #6 The participant identity and join path are reusable by M4-05 blocking and M4-06 admission without reworking the participant model.
- [ ] #7 A second concurrent device is allowed: one participant holds multiple connections, and the participant is present while at least one connection is live.
- [ ] #8 Removing or muting a participant resolves unambiguously across that participant's connections, and no duplicate participant row is created by a second device.
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
Implemented the domain participant, migration 0006 with the partial unique index, the ParticipantStore port and Postgres adapter, the join/leave use cases over one deny-by-default guard, and the WebSocket join path with a connection-scoped generation.

Acceptance criteria 1-5 are met and tested (unit, Postgres integration, and WebSocket integration). Criteria 6-8 are deliberately left unchecked:
- #6 (reusable by M4-05/M4-06): the model is designed for reuse and M4-06 layers admission on the same require_joinable guard, but nothing consumes it yet, so reuse is a claim rather than a proven fact. M4-05/M4-06 will close it.
- #7 (presence while >=1 connection live): a second device is allowed and reuses the identity (tested), but explicit presence counting was removed because nothing consumed it; leaving dead code would fail the workspace lints and the repo rule against speculative abstractions. Presence counting lands with its first real consumer.
- #8 (removal/mute across connections): the no-duplicate-row half is proven by the partial unique index. The cross-connection targeting half belongs to M4-05, which adds the send-to-actor seam when it needs it.

Generation is connection-scoped and replica-local. The durable uniqueness guarantee (no duplicate on reconnect) comes from the database index, not the generation.

Cross-replica coordination is NOT covered by M1-06: TASK-7 is Done and delivered only the Redis adapter (CoordinationStore, pub/sub, TTLs, ADR 0017), never wiring connection state onto it. The connection hub that would make generation ordering, session roster fan-out, and presence counting replica-safe is TASK-34 (M5-03 Realtime fanout), whose AC 1 is 'clients on two API replicas receive committed message once'. TASK-34 must treat the session-scoped registry methods added here as in scope for that work. Tracked as TASK-133.

Validation: cargo fmt --check clean, clippy --workspace --all-targets -D warnings clean, 150 tests pass across 35 suites including the Postgres-backed participant and migration tests, which now run under MAHFEL_TESTCONTAINERS=1 and also via MAHFEL_TEST_DATABASE_URL. just docs builds. sqlfluff reports only the pre-existing 0004_identity.sql LT02, which is left alone to preserve the SQLx checksum.
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
