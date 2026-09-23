---
id: TASK-132
title: M4-16 Session scheduling and title
status: Done
assignee: []
created_date: '2026-09-20 19:44'
updated_date: '2026-09-23 02:04'
labels:
  - backend
  - api
milestone: m-4
dependencies:
  - TASK-26
  - TASK-124
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 140000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Give a session an optional single-occurrence schedule and an optional custom title. A session can be ad-hoc (no schedule) or scheduled with an optional start instant and an optional duration; both are optional and the duration is advisory only, never auto-ending a session. The session title is optional and falls back to the room title when unset; the room title stays required. These fields extend the sessions table created by the amended migration 0002 rather than adding a separate schedule table. Setting the schedule or title is a management action gated by capability and audited.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A session may be ad-hoc or scheduled with an optional scheduled start instant and an optional positive bounded duration, both validated in the domain and by the database.
- [x] #2 Session title is optional and validated like a room title, and read responses fall back to the room title when the session title is unset.
- [x] #3 The schedule never constrains start: a capable principal can start an ad-hoc or scheduled session at any time.
- [x] #4 Duration is advisory and never auto-ends a live session; only an explicit end transitions it.
- [x] #5 Setting or changing the schedule and title requires the manage capability for the target and is persisted and audited.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add optional validated session title and 1..10080 minute advisory duration; retain optional scheduled_start_at and permit detail edits in every lifecycle state, including ended/cancelled. 2. Add nullable title/duration columns with CHECK constraints in additive migration 0015 and update Postgres session mapping. 3. Extend create and add PATCH session details with omitted=unchanged and null=clear, gated by manage_session and audited. 4. Resolve effective response title from the current room title when unset; expose whether a custom title exists. 5. Update OpenAPI/generated client and architecture docs. 6. Cover validation, persistence, fallback, authorization, all-state edits, and lifecycle independence with focused checks.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Decision: title and advisory schedule details remain editable in scheduled, live, ended, and cancelled states. Duration accepts 1..10080 minutes and never triggers lifecycle transitions. PATCH uses omit=keep/null=clear; PostgreSQL detail updates do not overwrite concurrent lifecycle columns.

Validation: 324 domain/application/API tests passed; PostgreSQL Testcontainers migrations and sessions tests passed; backend fmt and lint (including SQLFluff), frontend format and typecheck, strict docs build, OpenAPI export/client generation, and git diff --check passed.

Final review tightened PATCH persistence: the store now writes only fields present in the request, preserving independent concurrent detail edits as well as lifecycle transitions. PostgreSQL Testcontainers coverage exercises both races; 324 domain/application/API tests and backend lint/fmt passed again.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added optional validated session title and advisory duration with additive migration, capability-gated create/start/detail updates, room-title fallback in responses, audit events, and regenerated REST types. Verified focused backend, PostgreSQL integration, frontend contract, lint, and docs checks.
<!-- SECTION:FINAL_SUMMARY:END -->
