---
id: TASK-26
title: M4-01 Session vocabulary and lifecycle
status: Done
assignee:
  - '@opencode'
created_date: '2026-09-08 00:31'
updated_date: '2026-09-20 22:37'
labels:
  - backend
  - domain
  - api
milestone: m-4
dependencies:
  - TASK-3
  - TASK-15
  - TASK-22
documentation:
  - docs/backlog/roadmap.md
priority: high
ordinal: 34000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Rename the live aggregate from ClassroomSession to the generic Session and implement the session lifecycle that anchors M4. A persistent Room can host sessions; a Session has an explicit state machine (scheduled to live to ended, or scheduled to cancelled), at most one live session per room, ownership by a room manager or service principal, and start/end/cancel APIs. Because the image is not deployed beyond local development, migration 0002 is amended in place (classroom_sessions to sessions) and the required local database reset is documented.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Domain, application, PostgreSQL, and API code use Session and sessions, and no ClassroomSession or classroom_sessions identifier remains in backend production code or schema.
- [x] #2 Invalid transitions are rejected with deliberate errors and leave state unchanged: start from non-scheduled, end from non-live, cancel from live or ended.
- [x] #3 At most one live session per room holds under concurrent starts, and the losing request receives a conflict rather than a duplicate live session.
- [x] #4 Start, end, and cancel persist and produce audit events, and ownership is restricted to room managers and service principals deny-by-default.
- [x] #5 Migration tests cover the renamed schema on a fresh database, and documentation states the amended-0002 checksum consequence and local reset.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Amend migration 0002: rename classroom_sessions -> sessions, rename indexes, add partial unique index sessions_one_live_per_room_idx on (room_id) WHERE status='live'.
2. Domain: rename ClassroomSession->Session, ClassroomSessionId->SessionId; keep SessionStatus/SessionError and invariants; add storage-name test.
3. Application: rename types; add list_for_room to SessionStore; add actor field to commands; each use case takes R: SessionStore + S: RoomStore, calls require_manager, emits audit tracing after save; map StoreError::Conflict to typed Conflict (409). Add list/get use cases.
4. Postgres: rename table/index SQL; map 23505 to StoreError::Conflict; implement list_for_room; update migrations test + add sessions integration test.
5. API: add sessions route module with 6 endpoints, DTOs, utoipa registration, REST_PATH_TEMPLATES; add SessionStoreRef dyn seam on AppState; wire PostgresSessionStore; add API tests.
6. Docs: components.md + data-flow.md lifecycle wording; getting-started local reset/checksum note.
7. Validate: fmt, clippy, workspace tests with disposable DB, just docs. Update TASK-26.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implementation: amended migration 0002 in place (classroom_sessions -> sessions, indexes sessions_room_id_idx/sessions_status_idx, new partial unique index sessions_one_live_per_room_idx on (room_id) WHERE status='live'). Domain rename ClassroomSession/ClassroomSessionId -> Session/SessionId (module session.rs, SessionStatus/SessionError kept). Application: SessionStore gains list_for_room; Schedule/Start/End/Cancel commands carry actor; each use case takes R: SessionStore + S: RoomStore and calls crate::rooms::require_manager; store 23505 -> typed Conflict (409); invalid domain transition -> Conflict; audit tracing target=audit after successful save. Added manager-scoped list/get use cases. Postgres: sessions table/index SQL, sqlx_store_error now maps 23505 -> Conflict, list_for_room; migrations test asserts sessions table, renamed indexes, partial unique index; new tests/sessions.rs covers conflict + list_for_room + concurrent race. API: routes/sessions with POST/GET /v1/rooms/{room_id}/sessions and GET/start/end/cancel /v1/sessions/{session_id}..., DTOs + utoipa paths/schemas + REST_PATH_TEMPLATES; AppState gains a small object-safe SessionStoreRef seam wired from PostgresStores.sessions. Docs: components.md lifecycle wording, data-flow.md session lifecycle flow, getting-started.md amended-0002 checksum/local-reset note.

Validation: cargo fmt --all --check OK; cargo clippy --workspace --all-targets clean; cargo test --workspace with MAHFEL_TEST_DATABASE_URL against a disposable database on the local Postgres passes (migrations_provision_a_disposable_database, service_principal_lifecycle_and_room_ownership, sessions_enforce_one_live_per_room incl. concurrent race, API 70 tests incl. 6 new session tests); sqlfluff lint migrations clean for 0002 (pre-existing 0004 LT02 only); just docs (mkdocs --strict) OK. Testcontainers not used: MAHFEL_TEST_DATABASE_URL provided, as in prior tasks.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Renamed the live aggregate to the generic Session (table mahfel.sessions, amended migration 0002) and delivered its lifecycle: explicit state machine, one-live-session-per-room enforced by a partial unique index, manager-only schedule/start/end/cancel (service principals reach it by being room managers), typed 409 conflicts, structured audit logs, and REST endpoints with OpenAPI coverage. Verified with fmt, clippy, workspace tests on a disposable PostgreSQL database, and mkdocs strict.
<!-- SECTION:FINAL_SUMMARY:END -->
