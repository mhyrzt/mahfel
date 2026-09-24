---
id: TASK-139
title: Use typed TOML fixtures for development data
status: Done
assignee:
  - '@mahyar'
created_date: '2026-09-24 20:18'
updated_date: '2026-09-24 21:52'
labels:
  - backend
  - seed
dependencies: []
ordinal: 147000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Make mahfel-api seed load validated TOML fixture files instead of SQL and a separate JSON user fixture. Preserve idempotent development identities, and add stable room and scheduled-session fixtures for manual and E2E testing, including standalone sessions for existing rooms.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Default seed loads bundled TOML fixtures in filename order and preserves development principal and Keycloak user outcomes
- [x] #2 An explicit TOML file or directory is accepted; malformed or unsupported fixtures fail before writes
- [x] #3 No seed file is executed as SQL; existing SQL and JSON fixtures are removed
- [x] #4 Focused tests and deployment documentation describe the new fixture format and command behavior
- [x] #5 Room fixtures create stable rooms, memberships, and scheduled sessions idempotently for manual and E2E testing
- [x] #6 Seed composition, file loading, identity fixtures, and room/session fixtures live in focused modules with seed.rs as a small entrypoint
- [x] #7 TOML fixtures may reference relative or absolute local files and seed verified ready room, session, and avatar assets idempotently
- [x] #8 Ready seeded avatars appear in authenticated profile and live session roster views
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Inspect file storage and avatar contracts. 2. Add typed file fixtures with path resolution and preflight validation. 3. Upload through S3 and complete through existing verification, with stable IDs and idempotent reruns. 4. Document fixture behavior, validate storage/database integration, then create split commits containing only our changes.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
All selected TOML fixtures are parsed and domain-validated before connecting. Principal and room writes are parameterized; room, membership, and nested-session writes are atomic. Standalone sessions target existing rooms. Existing room owners and session room references are checked on rerun; seeded sessions remain scheduled and state is preserved.

Validation: API seed parser tests 3 passed; disposable PostgreSQL sessions integration test passed (including repeated room and standalone session seeding); backend fmt, workspace lint, mahfel-api build, strict docs build, and git diff --check passed. Full Keycloak plus API seed command was not run against a local stack.

Refactored the 469-line API seed module into a 59-line entrypoint plus files, fixture, identity, and rooms modules. Runtime behavior and fixture formats remain unchanged. Validation for this refactor: 3 focused API seed tests, backend format, workspace lint, mahfel-api build, and git diff --check passed. PostgreSQL write paths were not changed by this refactor.

Local asset paths resolve relative to their explicit TOML file or remain absolute. File bytes are preflight-verified, uploaded through the internal S3 endpoint, then completed by the existing application verification flow. Ready avatar URLs are signed for authenticated profile and session roster responses.

Validation for file and avatar extension: 5 API seed tests, OpenAPI route coverage, backend fmt and workspace lint, frontend lint/typecheck/format, 20 frontend tests, and strict docs build passed. Full API test run passed 141 tests and failed 2 unrelated WebSocket authentication tests. Frontend production build was blocked by sandbox Turbopack port binding (EPERM). Live Keycloak plus PostgreSQL plus RustFS seed flow was not exercised.

RustFS Testcontainers integration passed with MAHFEL_TESTCONTAINERS=1: seed object PUT, read-back bytes and MIME, and the existing direct transfer contract.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Replaced SQL and JSON seeding with validated TOML fixtures for identities, rooms, sessions, and local file assets; split seed code into focused modules. Seeded avatars appear in profile and live roster views. Parser, RustFS integration, route, frontend, lint, and docs checks passed. A full Keycloak/PostgreSQL/RustFS seed run remains unverified locally.
<!-- SECTION:FINAL_SUMMARY:END -->
