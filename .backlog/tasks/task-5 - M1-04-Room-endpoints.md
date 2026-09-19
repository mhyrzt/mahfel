---
id: TASK-5
title: M1-04 Room endpoints
status: Done
assignee:
  - '@mahyar'
created_date: '2026-09-08 00:31'
updated_date: '2026-09-19 15:54'
labels: []
milestone: m-1
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 13000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: a vertical durable slice is required before richer features, and the room model needs visibility, membership, and room-scoped roles before sessions, chat, and media attach to it.

A Room is a durable container. Identity is the room id only; titles may duplicate. A room is public or invite-only. Membership is (room, user, role) and is created automatically (auto join): a manager adding a user grants access immediately, and joining a public room via link grants access immediately. Roles are room-scoped: manager and member. Removing a membership removes access. Changing visibility preserves members and roles.

Implementation scope: rooms.visibility migration, room_memberships table, domain membership invariants (including last-manager protection), transactional create-with-membership, authorized create/get/list-joined/join/add-member/remove-member/update-settings handlers, API tests, removal of the duplicate legacy repositories.rs, and migration of the M1-02 integration test to Postgres*Store.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Room is created/read via API and remains after restart.
- [x] #2 GET /v1/rooms/{id} returns 404 both when the room does not exist and when the caller is not a member
- [x] #3 GET /v1/rooms lists only rooms the caller is a member of
- [x] #4 Room creation atomically persists the room and the creator's manager membership; created_by remains the owner reference
- [x] #5 A manager can add and remove members; the last manager cannot be removed
- [x] #6 POST /v1/rooms/{id}/join joins a public room idempotently and returns 404 for an invite-only room
- [x] #7 PATCH /v1/rooms/{id} lets a manager change title or visibility and preserves existing members and roles
- [x] #8 Rooms are public or invite-only; titles are not unique
- [x] #9 The duplicate legacy repositories.rs is removed and the M1-02 integration test uses Postgres*Store
<!-- AC:END -->



## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Domain: RoomVisibility, Room.visibility, RoomMember/RoomRole (manager|member), last-manager and transition invariants, unit tests.
2. Application: extend RoomStore and add membership operations; use cases create, get, list_joined, join, add_member, remove_member, update_settings. Room-scoped authorization lives in use cases that load membership; Authorizer stays for coarse actions.
3. Postgres: migration 0003 (rooms.visibility default invite_only, room_memberships with PK (room_id, user_id), role check, timestamps, user_id index); adapters; transactional create; delete repositories.rs; migrate M1-02 integration test.
4. API: POST /v1/rooms, GET /v1/rooms, GET /v1/rooms/{id}, POST /v1/rooms/{id}/join, POST /v1/rooms/{id}/members, DELETE /v1/rooms/{id}/members/{userId}, PATCH /v1/rooms/{id}; DTOs and error mapping (422/404/409).
5. Tests: domain, application, API (401/404/409/manager paths), Postgres integration, and the restart-durability proof for AC #1.
6. Validation: just backend fmt/lint/test/build, sqlfluff, just docs, docker compose config.
7. Commits in focused slices.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Domain: RoomVisibility (public|invite_only), RoomRole (manager|member), RoomMember, ensure_member_removable (last-manager rule). Room::create now takes visibility and Room::set_visibility preserves members. Titles are not unique; identity is the room id only.

Application: extended RoomStore with create_with_manager (transactional), list_for_member, get_member, list_members, add_member, remove_member (atomic last-manager guard). Use cases create_room, get_room, list_joined, join_room, add_member, remove_member, update_settings; room-scoped authorization lives in the use cases (membership load) while Action::CreateRoom goes through the authorization seam. Manager-only actions return 404 for non-members and 403 for members without the manager role.

Persistence/API: migration 0003 adds rooms.visibility (default invite_only) and mahfel.room_memberships (PK (room_id, user_id), role column named member_role because role is a SQL keyword, both FKs cascade, updated_at >= created_at). Endpoints: POST /v1/rooms, GET /v1/rooms (joined only), GET /v1/rooms/{id} (404 missing or non-member), POST /v1/rooms/{id}/join (public idempotent, invite-only 404), POST/DELETE /v1/rooms/{id}/members[/{userId}], PATCH /v1/rooms/{id}. Deleted the duplicate legacy repositories.rs and moved the M1-02 integration test to Postgres*Store.

Validation: just backend lint clean (clippy -D warnings + sqlfluff); cargo test --workspace --all-features --lib --bins passes (api 29 incl. room auth/404/409 paths, domain 10); real-Postgres integration test passed against a disposable container via MAHFEL_TEST_DATABASE_URL (3 migrations, idempotent re-run, single _sqlx_migrations, membership create/read, restart durability); mahfel-api migrate ran twice on a fresh DB then serve reached 'API service ready'; just docs strict pass; docker compose config OK.

Known limitation: the restart-durability proof is the PostgreSQL integration test plus API-level create/get tests; a full HTTP create -> process restart -> HTTP get end-to-end was not run because this sandbox intercepts loopback HTTP. Testcontainers itself remains blocked in this environment, so the disposable-container path (MAHFEL_TEST_DATABASE_URL) was used instead.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Shipped the room vertical slice: visibility (public|invite_only), room-scoped memberships with manager/member roles and last-manager protection, transactional create-with-manager, joined-only listing, idempotent public join, and manager add/remove/update, all behind the M1-07 authorization seam. GET returns 404 for missing or non-member to avoid leaking existence. Migration 0003 adds visibility and room_memberships; the duplicate repositories.rs was deleted and the M1-02 test now uses Postgres*Store. Verified with clippy, sqlfluff, workspace tests, a real-Postgres integration test, and an API boot on a freshly migrated database.
<!-- SECTION:FINAL_SUMMARY:END -->
