---
id: TASK-114
title: M2-09 Service principals
status: Done
assignee:
  - '@mahyar'
created_date: '2026-09-19 12:34'
updated_date: '2026-09-20 21:57'
labels: []
milestone: m-2
dependencies:
  - TASK-9
  - TASK-11
  - TASK-15
documentation:
  - docs/adr/index.md
priority: medium
ordinal: 122000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: external systems such as lms-core must create, populate, and manage rooms through the API, but the identity model is user-only (Principal is a UserId and rooms.created_by references users). Today a service cannot be an owner or manager without impersonating a human user.

Implementation scope: a first-class machine actor distinct from users, its authentication mechanism (issuer-issued client-credentials/service-account token or an equivalent documented mechanism), provisioning and revocation, and room ownership/management by a service principal. This is deferred from M1-04, which keeps rooms user-owned.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A service principal authenticates as a first-class actor distinct from any user row, and Mahfel never represents a service as an impersonated user
- [x] #2 A service principal can create rooms and manage members and settings exactly like a user-owned room, and service-authored rooms record the service principal as owner
- [x] #3 Service principals can be provisioned, rotated, and revoked; revocation rejects subsequent requests
- [x] #4 Authorization matrix tests cover service principals next to users and stay deny-by-default
- [x] #5 An ADR records the chosen service-authentication mechanism, including how local development provisions a service principal
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Forward migration 0005: mahfel.actors (user/service), backfill from users, users.id FK -> actors, service_principals, service_api_keys, rooms.created_by -> actors, rename room_memberships.user_id -> actor_id.
2. Domain: new actor.rs (ActorId, ActorKind, ServicePrincipal, ServicePrincipalStatus), rename UserId -> ActorId across backend, RoomMember.user_id -> actor_id, lib re-exports.
3. Application: sha2 dep, ports/service_principal_store.rs, service_principals/ module (api_key, create, rotate, revoke, authenticate, list), Principal actor_id+kind, RoleBasedAuthorizer docs/tests.
4. Postgres: stores/service_principal.rs, user/room store actor-aware, wire PostgresStores, integration test, migrations test update.
5. API: AppState service_auth, http/auth ServiceKeyAuthenticator adapter, extract API-key path, principal.rs actor_id+kind, auth claims app_principal_kind, admin /v1/admin/service-principals routes + OpenAPI, serve/http wiring, tests.
6. Docs: ADR 0022 + index + mkdocs, components.md, backend/AGENTS.md.
7. Validate fmt/lint/test; update Backlog.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Migration 0005_actors.sql: mahfel.actors (kind user|service), backfill one user actor per existing user, users.id FK -> actors, service_principals, service_api_keys, rooms.created_by -> actors, room_memberships.user_id -> actor_id. sqlfluff-clean (single RF04 noqa on the spec-mandated 'name' column).

Domain: new actor.rs (ActorId, ActorKind, ServicePrincipal, ServicePrincipalStatus); UserId renamed to ActorId everywhere; RoomMember.user_id -> actor_id; Identity.user_id stays named but typed ActorId.

Application: sha2 0.10, ServicePrincipalStore port, service_principals use cases (api_key generate/hash, create/rotate/revoke/authenticate/list), Principal now actor_id + kind. Raw keys returned once; Debug intentionally not derived on key-bearing types so they cannot be logged.

Postgres: PostgresServicePrincipalStore (single-transaction create; revoke revokes active keys; rotate_key locks+checks active then swaps keys; find_active_by_key_hash joins active key+principal and best-effort touches last_used_at). UserStore::insert and identity provisioning now create the actors row in the same transaction. Seed fixture inserts the actor first.

API: AppState gains service_auth (dyn ServiceKeyAuthenticator) and service_principals (dyn ServicePrincipalAdmin). Decision: used a small dyn admin seam instead of adding a second generic parameter to AppState, matching the existing RequestAuthenticator pattern. Authenticated tries the mhf_ API-key path first; VerifiedToken stays JWT-only. Admin routes /v1/admin/service-principals (POST create, GET list, POST /{id}/rotate, DELETE /{id}) all require ManageSystem and are in OpenAPI. app_principal_kind claim parsed; unknown/absent treated as user (documented).

Validation: cargo fmt --all --check OK; cargo clippy --workspace --all-targets --all-features -- -D warnings clean; cargo test --workspace with MAHFEL_TEST_DATABASE_URL against a disposable database on the local Postgres container passes (migrations_provision_a_disposable_database, service_principal_lifecycle_and_room_ownership, plus API/unit tests); sqlfluff lint migrations/0005_actors.sql clean; just docs (mkdocs --strict) OK. Testcontainers could not run: Docker socket returned connection reset (known environment issue), so integration tests used MAHFEL_TEST_DATABASE_URL on throwaway databases instead. Pre-existing, unrelated: sqlfluff 4.1.0 flags migration 0004 (unchanged; not edited to preserve its SQLx checksum).

Not implemented (out of scope): WebSocket API-key authentication; the WS gateway remains JWT-only.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Shipped service principals as first-class machine actors (ADR 0022). Migration 0005 adds mahfel.actors (user|service), service_principals, and service_api_keys, backfills user actors, and repoints rooms.created_by and room_memberships.actor_id at actors. Domain renames UserId to ActorId and adds ServicePrincipal. Application adds a ServicePrincipalStore port and create/rotate/revoke/authenticate/list use cases over opaque mhf_ keys stored only as SHA-256. Postgres implements the port transactionally and creates actor rows for users and identities. The API authenticates mhf_ keys through a dyn ServiceKeyAuthenticator, exposes admin provisioning under /v1/admin/service-principals behind ManageSystem, and parses app_principal_kind (default user). Verified with fmt, clippy -D warnings, cargo test --workspace against a real disposable Postgres database (migration + service-principal lifecycle + room ownership + API tests), sqlfluff on 0005, and mkdocs --strict.
<!-- SECTION:FINAL_SUMMARY:END -->
