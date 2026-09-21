---
id: TASK-123
title: M4-03 Participation records
status: Done
assignee: []
created_date: '2026-09-20 18:58'
updated_date: '2026-09-21 15:50'
labels:
  - backend
milestone: m-4
dependencies:
  - TASK-27
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 131000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Replace the LMS attendance concept with generic, durable participation records. A participant (M4-02) accumulates join/leave intervals per session; intervals aggregate to connected duration, are queryable per session and per participant, and can be corrected with an audited history. Participation is durable PostgreSQL state and is never derived from ephemeral Redis presence.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Every connect and disconnect opens or closes a participation interval, and intervals for one participant never overlap.
- [x] #2 Intervals aggregate to a per-participant total connected duration and are queryable by session and by participant.
- [x] #3 The reconnect grace policy is explicit and documented, and reconnects within the window do not fragment records.
- [x] #4 Corrections are audited and preserve the original interval history.
- [x] #5 No interval remains open after the session ends, including after abrupt disconnects or API restarts.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented durable participation records as the generic replacement for an attendance concept.

Domain (crates/domain/src/participation.rs). ParticipationInterval with open/close/supersede, IntervalError, CorrectionReason (closed vocabulary: wrong_end, wrong_start, erroneous, manual_adjustment), and two pure helpers: intervals_overlap (half-open, so a disconnect and immediate reconnect are adjacent not overlapping) and total_connected_seconds (excludes superseded, clamps negatives). is_open/superseded. Intervals are append-only: supersede marks the original instead of mutating its measurement, and superseding an open interval also closes it so it cannot satisfy the one-open-interval rule.

Migration 0007_participation.sql. mahfel.participation_intervals referencing session_participants (not a transport), with a partial unique index on (participant_id) WHERE ended_at IS NULL for the at-most-one-open-interval invariant, a CHECK that superseded_by and correction_reason are both present or both absent, a CHECK that ended_at >= started_at, and indexes for session and actor queries.

Port + adapter. ParticipationStore (open/close/close-for-session/get/list-by-participant/list-by-session/supersede) and PostgresParticipationStore. supersede_interval inserts the replacement before marking the original, in one transaction, because superseded_by references the replacement row.

Use cases (crates/application/src/participation/). open/close interval, close_session_intervals (the end-of-session sweep), session_participation (manager-gated summary), participant_participation (self-visible, manager-visible otherwise), correct_interval (manager-only, transactionally swaps original for replacement and audits with target audit).

Wiring. join_session now opens an interval on every accepted join, including a reconnect, and tolerates a conflict when an interval is already open (a reconnect after a missed disconnect is still a present participant). leave_session closes the open interval. end_session sweeps every open interval in the session after the session is durably ended. The WebSocket teardown closes the interval on every exit path (client close, heartbeat timeout, I/O error, shutdown), not only a graceful leave.

REST. GET /v1/sessions/{id}/participation (manager), GET /v1/sessions/{id}/participants/{pid}/participation (self or manager), POST /v1/participation/intervals/{id}/correct (manager). Registered in the route table, ApiDoc paths, and REST_PATH_TEMPLATES; the OpenAPI coverage test passes.

Decisions.
- Reconnect grace: none. A disconnect closes the interval immediately and a reconnect opens a new one against the same durable participant. The aggregate is exact and there is no clock-skew window or expiry job; records do fragment across a flaky connection.
- Open-interval cleanup: join-time tolerance plus the end-of-session sweep. No worker reconciliation job was added, so a replica that dies mid-session leaves intervals that the next session end or query closes; the documented guarantee is that no interval survives its session.
- Corrections supersede rather than overwrite, so the original measurement and the reason it changed both survive.

Two bugs found by the Postgres integration test and fixed: supersede marked the original before inserting the referenced replacement (FK violation), and the domain supersede left an open interval open, contradicting the one-open-interval invariant. Both are covered by tests now.

Transport timestamp format. time's default serde form for OffsetDateTime is a tuple, and its human-readable form uses a space separator with mandatory fractional seconds and offset seconds. Added bins/api/src/http/routes/rfc3339.rs and pinned the participation DTO timestamps to RFC 3339. This surfaced that the pre-existing scheduled_start_at field had no defined wire format and no caller; it is left as-is and out of scope for this task.

Validation. cargo fmt --check clean; clippy --workspace --all-targets -D warnings 0 errors; full workspace suite green (36 suites, 180 tests). Postgres integration under MAHFEL_TESTCONTAINERS=1 passes for migrations (count now 7), participants, participation, sessions, and service principals. just docs builds. sqlfluff reports only the pre-existing 0004_identity.sql LT02, which is left alone to preserve the SQLx checksum.
<!-- SECTION:NOTES:END -->
