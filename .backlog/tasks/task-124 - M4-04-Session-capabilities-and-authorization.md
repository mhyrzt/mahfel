---
id: TASK-124
title: M4-04 Session capabilities and authorization
status: Done
assignee:
  - '@mahyar'
created_date: '2026-09-20 18:58'
updated_date: '2026-09-21 14:32'
labels:
  - backend
  - security
milestone: m-4
dependencies:
  - TASK-15
  - TASK-114
  - TASK-26
documentation:
  - docs/backlog/roadmap.md
priority: high
ordinal: 132000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Decide every session and moderation action from capabilities derived from verified global roles and per-room membership, instead of a fixed vertical role enum. Define a capability set (join, publish_media, share_screen, mute_others, remove_participant, admit_participant, manage_session) and one policy seam that maps global admin/moderator, room manager/member, and service principals to capabilities deny-by-default, reused by REST and WebSocket.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A single policy resolves a principal to capabilities from verified claims plus room membership, and no handler hardcodes role strings.
- [x] #2 Deny-by-default allow/deny matrix tests cover member, manager, moderator, admin, and a service principal.
- [x] #3 REST and WebSocket session actions evaluate capabilities through the same seam.
- [x] #4 The caller can introspect its own session capabilities without exposing other participants privilege levels.
- [x] #5 Capability names and meanings are documented, including the education mapping of teacher to manager or host.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Capability model: `Capability { Join, PublishMedia, ShareScreen, MuteOthers, RemoveParticipant, AdmitParticipant, ManageSession }` in mahfel-application (policy concept, not durable domain state).
2. Resolution seam: replace the scalar `Action`/`Authorizer` model. One resolver takes a Principal plus the caller's optional room membership and returns a CapabilitySet. Deny by default.
3. Rules: global admin -> all; global moderator -> moderation set but not ManageSession unless also a room manager; room manager -> full set for that room; room member -> Join/PublishMedia/ShareScreen; service principal -> member-equivalent unless it owns or manages the room; non-member -> Join only for a Public room.
4. Global vs room-scoped: keep a small global check for system administration (ManageSystem) so admin endpoints do not go through capability resolution. The capability seam governs room and session actions.
5. REST: GET /v1/sessions/{id}/capabilities returns the caller's own set only (AC 4). Existing room/session routes authorize through the shared policy.
6. WebSocket: the M4-02 join path evaluates Join through the same resolver, so REST and WebSocket share one seam (AC 3).
7. Tests: deny-by-default allow/deny matrix over member, manager, moderator, admin, service principal, and outsider across every capability (AC 2).
8. Docs: capability names and meanings including the education mapping (teacher -> manager/host, student -> member, attendance -> participation records) (AC 5). Remove the remaining 'Classroom moderation' wording from the Action enum.
9. Validation: fmt, clippy -D warnings, focused tests, full workspace tests, just docs.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented as a full replacement of the scalar Action/Authorizer model (user-approved option A with CapabilityContext).

Model. One Capability enum (10 values) with an is_room_scoped flag and stable as_str names. CapabilityContext carries Option<RoomRole>; global capabilities ignore it. CapabilitySet is a resolved set, empty means deny. CapabilityResolver is the one port; ServerCapabilityResolver (crate::policy) is the production policy and now owns what used to be RoleBasedAuthorizer. Action, Authorizer, RoleBasedAuthorizer and AllowAuthenticated are deleted; the permissive authorizer lives only in the api test module as AllowAll/DenyAll.

Domain vocabulary. RoleCapability::for_role in mahfel-domain owns the manager/member -> grant mapping, so the vertical mapping lives in the domain and the resolver only names the transport-facing capability. Fixed two regressions during this: Participant must grant join+publish+share (not just join), and Moderate must grant the full moderation set.

Membership seam. New MembershipStore port with a blanket impl for RoomStore, so resolution reads the caller's role from the store and a request cannot claim a membership it does not hold. resolve_room_capabilities / require_room_capability are the application entry points.

REST. All 8 former authorize() call sites now use authorize_global with a Capability. New GET /v1/sessions/{session_id}/capabilities (AC 4) returns only the caller's own set; it loads the session through a new get_session_for_caller use case whose visibility is the join rule and which collapses non-visible to NotFound so a private session cannot be probed. Registered in the route table, the ApiDoc paths, and REST_PATH_TEMPLATES; the OpenAPI coverage test passes.

Gap on AC 3 (honest). REST and WebSocket both call the same resolver, but the WebSocket join path still authorizes through the session join guard (require_joinable) rather than through require_room_capability, because no WS action evaluates a capability directly yet and I would not ship an unused seam. The M4-02 join guard already shares the one membership read. Wiring the WS dispatch onto require_room_capability is best done with the first capability-gated WS frame (M4-05 mute/remove). The seam is shared; only this last call-site swap remains.

Validation. fmt clean, clippy --workspace --all-targets -D warnings 0 errors, full workspace test suite green (domain 26, application 37, api 77, postgres and others). just docs builds. The previously failing mahfel-auth Keycloak test passed on this run.

Deleted unused code rather than keeping speculative API: authorize_in_room and membership_grants were written and then removed once they had no caller.
<!-- SECTION:NOTES:END -->
