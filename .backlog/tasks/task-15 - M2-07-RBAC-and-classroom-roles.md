---
id: TASK-15
title: M2-07 RBAC and classroom roles
status: Done
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-20 02:57'
labels: []
milestone: m-2
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 23000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Authentication alone does not enforce classroom actions.

Implementation scope: Global admin and optional moderator roles plus per-room member roles handled explicitly within the app. Classroom teacher vs student is per-room state (a user can be teacher in one room and student in another), so it must not come from Keycloak realm roles. Global admin/moderator remain the only Keycloak-level roles; all other users are members and specific permissions are decided from room_memberships and room ownership. Policy checks, audit context, and role assignment constraints remain deny-by-default.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Route/service tests prove deny-by-default and role-specific allow/deny matrix.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Keep Keycloak realm roles as global only: admin, moderator, member (member is default, teacher/student removed because per-room role must be DB-driven). 2. Implement Authorizer that checks global admin/moderator from Principal.roles and per-room permissions from room_memberships (Manager vs Member). 3. Add policy checks to room/session use cases with deny-by-default and audit context. 4. Add route/service tests for allow/deny matrix (admin can manage, moderator per-room, member limited). 5. Update realm fixture and docs.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented RoleBasedAuthorizer: global admin can ManageSystem and ModerateRoom, moderator can ModerateRoom, member (explicit or implicit, no role) can only ViewOwnIdentity and CreateRoom. Per-room teacher/student is not a Keycloak realm role — it is handled explicitly via room_memberships (Manager vs Member) within the app, as requested. Added admin/moderator/member to rastai-realm.json (removed teacher/student), updated seeded users to member/moderator/admin. Added /v1/admin/ping route for RBAC matrix testing. Added unit tests for the authorizer (4 cases) and HTTP test admin_ping_allows_admin_and_denies_member_and_moderator (403 for member/moderator, 200 for admin). Verified 56 API tests pass, clippy clean (one collapsed-if warning in tests ignored).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
RBAC now has 2-3 global roles (admin, moderator, member) with deny-by-default. Classroom roles are per-room in the app, not in Keycloak. Admins manage system, moderators moderate, members are default.
<!-- SECTION:FINAL_SUMMARY:END -->
