---
id: TASK-130
title: M4-14 Link join and approval flow
status: Done
assignee: []
created_date: '2026-09-20 19:31'
updated_date: '2026-09-22 01:54'
labels:
  - backend
  - api
  - security
milestone: m-4
dependencies:
  - TASK-129
  - TASK-125
  - TASK-124
  - TASK-27
documentation:
  - docs/backlog/roadmap.md
priority: high
ordinal: 138000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Redeem an invite link and drive the confirmation flow. An authenticated principal that redeems a valid link enters a waiting state unless it already holds explicit membership in the target. A principal with the admit capability approves or rejects the request. Approval grants session participation for a session link, and durable room membership for a room link, after which future joins bypass confirmation. Reject terminates the waiting state. This reuses the admission seam and the capability seam rather than adding a parallel mechanism.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Redeeming a valid link returns the resolved target and a waiting state unless the principal is already an explicit member of the target.
- [x] #2 Approving a session link admits the principal to that session only and creates no durable room membership.
- [x] #3 Approving a room link creates durable room membership, and subsequent joins to that room bypass confirmation.
- [x] #4 Approve and reject require the admit capability, reject ends the waiting state, and no media or publish capability is granted before approval.
- [x] #5 Invalid, expired, or revoked codes and repeated failed attempts return deliberate non-enumerating errors, are rate-limited, and are audited.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented invite-link redemption and the approval flow, connecting TASK-129's codes to TASK-125's lobby.

Domain. AdmissionTarget {Session, Room} replaces the required session_id on AdmissionRequest, so a request carries exactly one target as a closed enum and the session form exposes session_id(). The scope decides what approval grants, which is why it is modelled rather than inferred.

Migration 0010. admission_requests gains room_id (backfilled from each request's session before NOT NULL lands) and session_id becomes nullable with a CHECK that room_id is always present. The room is always recorded because authorization resolves against it for both scopes; a session id distinguishes the scopes. The two partial unique indexes and queue indexes are now scoped per form, so a repeated redemption against the same target cannot queue twice and a manager reads one queue per session and per room.

Ports and adapter. AdmissionStore methods key on AdmissionTarget rather than a session; get_pending and count_pending use IS NOT DISTINCT FROM so a room target matches NULL session correctly. list_pending_for_room reads the room-scoped queue. expire_pending_for_session deliberately ignores room-scoped rows.

Redemption (crates/application/src/invite/redeem.rs). Order of operations is the security property: the rate limiter runs before the code is read, so a limited attempt is indistinguishable from an invalid one; then the code is normalized; then a single resolve_usable_target collapses a malformed code, an unknown code, an expired or revoked link, a target whose row is gone, and an archived room into one identical error. Redemption never joins: it resolves and queues, and joining stays the join path's job. An existing member is returned as already_member rather than queued.

Approval semantics. admit now branches on the target. A session request requires the session to be live, creates the participant and opens the participation interval. A room request requires no live session at all, because room membership outlives any one session and that is what makes a room link durable onboarding; it grants RoomRole::Member, fixed rather than taken from the request, so a link can never confer management. reject mirrors both. Authorization is require_manager against the target's room either way.

Rate limiting. AttemptLimiter gains its first caller, and AppState.code_index is now read (its dead_code expectation is gone). WsState.code_index was removed instead: redemption is REST-only, so the field had no reader and keeping it would have been speculative.

Source address. New SourceAddressResolver with a configured trusted_proxy. X-Forwarded-For is honoured only when the socket peer matches the configured proxy; otherwise the header is ignored and the peer is used, because trusting it unconditionally would let any client forge a fresh budget. The unparseable-leading-entry case stops the chain rather than skipping it, so a forged list cannot be walked past. New ApiSettings.trusted_proxy, unset by default; config/docker.toml sets it to caddy.

REST. POST /v1/invite-links/redeem (code in the body, never the path, because paths are recorded by access logs and Referer) and GET /v1/sessions/{session_id}/admission for a waiting caller to read its own request. Both registered in ApiDoc and REST_PATH_TEMPLATES.

Audit. invite.redeem records link id, target kind, target id, and outcome. The raw code appears in no log line, and the non-enumerating response does not echo the submitted code.

Decisions (user-approved). Redeem returns a status and does not join. A room link with no live session still queues and is still approvable. The code travels in the body. A trusted proxy config gates forwarded-header trust rather than trusting it blindly. Membership means a room_memberships row only, so a pending request is not membership. Session approvals require live; room approvals do not. Room-link approval grants Member, never Manager.

Two bugs found and fixed during implementation. The 0010 CHECK initially required exactly one of session/room to be present, which rejected every legitimate session request, since those correctly carry their room as well; corrected to require room_id always and let session_id distinguish the scope. A test fixture re-seeded a revoked link while building the expired case, making the revoked branch actually active; the fixture now builds each unusable link without leaving a live one behind.

Validation. cargo fmt --check clean; clippy --workspace --all-targets -D warnings 0 errors; full workspace suite green (38 suites; domain 62, application 68, api 108). Postgres integration under MAHFEL_TESTCONTAINERS=1 passes for admission (migration count now 10), invite, migrations, participants, participation, sessions, and service principals. just docs builds. sqlfluff passes on 0010 after an alias named session was renamed, and still reports only the pre-existing 0004_identity.sql LT02.
<!-- SECTION:NOTES:END -->
