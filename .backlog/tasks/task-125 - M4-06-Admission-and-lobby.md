---
id: TASK-125
title: 'M4-06 Admission, lobby, and session access overrides'
status: Done
assignee: []
created_date: '2026-09-20 18:58'
updated_date: '2026-09-22 00:19'
labels:
  - backend
milestone: m-4
dependencies:
  - TASK-27
  - TASK-124
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 133000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Generic admission control replacing the LMS waiting room. A room carries an admission policy of open or lobby. In lobby mode a joining principal enters a bounded waiting state and cannot publish or consume until a principal with admit_participant admits or rejects it. Policy is enforced in join authorization, not only in the UI, and decisions are audited.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Room admission policy supports open and lobby and persists through the API.
- [x] #2 In lobby mode a joining principal is waiting and cannot publish or consume media until admitted.
- [x] #3 Admit and reject require the admit_participant capability, and reject terminates the waiting state.
- [x] #4 Admission is enforced in the join path for both REST and WebSocket, not only in the UI.
- [x] #5 Admission decisions are audited and the waiting set is bounded and cleared when the session ends.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented generic admission control, the lobby, and session access overrides (scope addition, user-approved).

Scope note. TASK-125 as written specified a room-only policy. While planning it, a concrete product shape surfaced that room-only policy cannot express: a public room whose particular session is locked down to invited people. ACL 1-5 are still met as written; session-level overrides were added as an explicit, user-approved extension, and the task was retitled to say so.

Domain. AdmissionPolicy {open, lobby} with requires_approval and a stricter() combinator. Room gains admission (default Open) and set_admission, guarded against an archived room like set_visibility. Session gains optional visibility and admission overrides plus effective_visibility(room) / effective_admission(room); None means inherit. SessionVisibility is a distinct type from RoomVisibility so the two axes cannot be assigned across. An override may only tighten: set_visibility/set_admission reject a value looser than the room's with SessionError::CannotLoosen, so a session can never expose a room the room itself protects. AdmissionRequest is state-checked (admit/reject/expire) and refuses a repeat decision. The never-loosen rule and all None/Some resolution combinations are unit-tested.

Migration 0009. rooms.admission NOT NULL DEFAULT open with a CHECK; sessions.visibility and sessions.admission nullable with CHECKs, where NULL is the inheritance signal; admission_requests with a partial unique index on (session_id, actor_id) WHERE status = 'pending' (mirroring the participants pattern) so a duplicate join cannot queue twice, a CHECK tying each status to the decision fields it may carry, and an origin column (join | invite_link) added now so M4-14 needs no migration. Migration count assertion 8 to 9.

Ports and adapters. AdmissionStore (insert, get_pending, get, list_pending, count_pending, decide_if_pending, expire_pending_for_session) and PostgresAdmissionStore. decide_if_pending is conditional so two concurrent decisions cannot both report success. The room and session row mappers were updated for the new columns; the integration test asserts the values round-trip so a mapper omission cannot silently return defaults.

Use cases. set_room_admission, set_session_access, view_session_access, view_lobby, admit, reject, enqueue_admission, wait_for_admission. Admission is capability-gated through require_manager with no new capability vocabulary; AdmitParticipant already existed from M4-04. admit creates the participant and opens the interval before marking the request decided, so a failure leaves the request pending and retryable rather than reporting a decision that granted nothing.

Join path. join_session now returns JoinOutcome::{Joined, Waiting} instead of a bare Participant. Authorization is unchanged and runs first; the effective policy then decides. Under lobby the caller gets a pending request, no participant, no interval, and no participation capability, so cannot-publish-until-admitted is structural. enqueue_admission is idempotent per principal, so a reconnect while waiting keeps its place, and the per-session cap (MAX_PENDING_PER_SESSION = 50) bounds a queue any authorized caller could otherwise fill. end_session retires pending requests alongside the participation sweep; an expiry records no deciding principal because none made the call.

Transport. GET/PUT /v1/rooms/{room_id}/admission, GET/PUT /v1/sessions/{session_id}/access, GET /v1/sessions/{session_id}/lobby, POST .../lobby/{actor_id}/admit and .../reject. All registered in the route table, ApiDoc paths, components schemas, and REST_PATH_TEMPLATES; the OpenAPI coverage test passes. RoomResponse and SessionResponse gained the policy and override fields. Over the WebSocket a deferred join is answered with a new join_waiting frame instead of joined. AppState gained admissions; the WS store bundle gained it too.

Decisions.
- Waiting state is durable PostgreSQL, not Redis. The waiting principal is looking at a waiting screen, so the request is user-visible and AGENTS.md forbids Redis as the sole source of such a fact. Whether the waiter is still connected stays ephemeral in the registry.
- Policy lives on both room and session, with the session inheriting by default and only ever tightening.
- Approving a session link grants that session only; this task supplies the lobby M4-14 will feed from.

Contract note. Deciding a request that is not waiting returns 409, not 404: a manager can already see the session, so it is a state conflict, and 404 stays reserved for a session the caller cannot see. The transport test was corrected to match rather than the mapping being changed.

Validation. cargo fmt --check clean; clippy --workspace --all-targets -D warnings 0 errors; full workspace suite green (38 suites; domain 60, application 53, api 98). Postgres integration under MAHFEL_TESTCONTAINERS=1 passes for admission, migrations (count now 9), invite, participants, participation, sessions, and service principals. just docs builds. sqlfluff reports only the pre-existing 0004_identity.sql LT02, left alone to preserve the SQLx checksum.
<!-- SECTION:NOTES:END -->
