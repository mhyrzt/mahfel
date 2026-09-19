---
id: TASK-8
title: M1-07 Authorization/OpenAPI decisions
status: In Progress
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-19 12:46'
labels: []
milestone: m-1
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 16000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Later routes need consistent policy and discoverability.

Implementation scope: Authorization service interface, deny-by-default convention, development principal, ADR/decision for OpenAPI generation.

Dependencies: M1-03
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Room handler uses authorization seam; development identity is explicit; OpenAPI approach recorded.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add portable authorization seam to mahfel-application (Principal, Action, AuthorizationError, Authorizer, interim AllowAuthenticated policy). 2. Add migration 0003 seeding an explicit development principal user; keep it inert unless config enables it. 3. Add api.development_principal_user_id config (unset by default; enabled in local-only compose/env examples). 4. Add API AppState + Authenticated extractor (401 when absent) and an authorize() helper (403 on denial); expose GET /v1/me demonstrating the seam. 5. Tests: 401 without principal, 200 with principal, 403 when policy denies. 6. Defer OpenAPI recording; keep TASK-8 in progress until TASK-5 wires the room handler to this seam.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Delivered authorization seam + explicit development principal (OpenAPI recording deferred at user request). mahfel-application gains ports/authorization.rs (Principal, Action, AuthorizationError, Authorizer, interim AllowAuthenticated policy). mahfel-api gains AppState + AuthState, an Authenticated extractor that returns 401 when no identity is configured, an authorize() helper mapping denials to 403, and GET /v1/me demonstrating the seam; routes now take AppState. Migration 0003 seeds a clearly labeled development principal, inert unless MAHFEL_API__DEVELOPMENT_PRINCIPAL_USER_ID is set (local compose/local.toml only; unset by default). Tests: 401 without principal, 200 with principal, 403 when policy denies (17 API tests). Also fixed stale mahfel-database package name in backend/justfile test-database.

Seed separation: reverted the migration seed; added backend/seeds/0001_development_principal.sql applied explicitly by a new 'mahfel-api seed' command. bins/api/build.rs embeds all seeds/*.sql at build time sorted by name (no runtime files), and the command accepts an optional file or directory path for applying one fixture. Added a one-shot mahfel-seed Compose service (migrate + seed) that mahfel-api depends on. Reverted the migration count assertion to 2.

Fixed a pre-existing migration integrity bug found while verifying seeds: the DB role and the 'mahfel' schema share a name and default search_path is "$user", public, so unqualified _sqlx_migrations resolved to public before migration 0001 created the schema and to mahfel afterwards, splitting bookkeeping and making a second migrate fail with 'relation users already exists'. Pinned search_path=public in mahfel-postgres::connect, added idempotent-migrate and single-tracking-table assertions to the Postgres integration test, and documented the rule in backend/AGENTS.md.

Validation: just backend lint clean (clippy workspace + sqlfluff); cargo test --workspace --lib --bins pass (api 17 incl. 401/200/403 identity paths); docker compose config OK; just docs strict pass. Manual verification against a disposable PostgreSQL container: migrate ran twice (second no-op), single _sqlx_migrations table with versions 1 and 2, seed idempotent across repeated runs, and the API boots to 'API service ready'. The Testcontainers migration test could not run in this environment (connection reset to the Docker socket/registry); the manual container run covers the same behavior. Remaining: TASK-5 must wire the room handler to the seam, and the OpenAPI decision is deferred by request.

TASK-5 (M1-04) now exercises this seam: room routes require the Authenticated extractor (401 without an identity) and POST /v1/rooms runs the Authorizer via Action::CreateRoom. Room-scoped membership checks live in the room use cases because they need the loaded membership; the Authorizer remains the coarse policy gate. The OpenAPI decision is still deferred by request, so this task stays In Progress.
<!-- SECTION:NOTES:END -->
