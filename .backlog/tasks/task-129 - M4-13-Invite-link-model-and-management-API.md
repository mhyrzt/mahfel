---
id: TASK-129
title: M4-13 Invite link model and management API
status: Done
assignee: []
created_date: '2026-09-20 19:31'
updated_date: '2026-09-21 21:09'
labels:
  - backend
  - api
  - security
milestone: m-4
dependencies:
  - TASK-26
  - TASK-124
documentation:
  - docs/backlog/roadmap.md
priority: high
ordinal: 137000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Introduce revocable, human-typable invite links for rooms and sessions, in the spirit of a Google Meet link. A link has a globally unique code in the form xxx-yyyy-zzz (lowercase a-z), a target (one room or one session), an expiry defaulting to 7 days and bounded by policy, a status of active, revoked, or expired, and a creator. Management API supports create, list, revoke, rotate, and regenerate for a room or session, gated by the manage capability. Codes are stored plaintext because they must be looked up directly, and are treated as bearer capabilities with rate-limited redemption, audit, and strict log hygiene.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Codes match the xxx-yyyy-zzz format, are globally unique, and are generated atomically with bounded collision retry; redemption normalizes mixed case and separators.
- [x] #2 A link targets exactly one room or one session and stores expiry, status, and creator; expired or revoked links are rejected on redemption.
- [x] #3 Create, list, revoke, rotate, and regenerate require the manage capability for the target and are deny-by-default.
- [x] #4 Rotate and regenerate invalidate the previous code atomically, and revoke is terminal.
- [x] #5 Redemption is rate-limited per principal and address, create and revoke and redeem emit audit events, and codes never appear in logs or metrics labels.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented the invite-link model and management API, plus the redemption primitives M4-14 will call.

Domain (crates/domain/src/invite.rs). InviteCode is a normalized value object: generate, parse, format, as_str. Generation rejection-samples bytes so every letter keeps equal probability; byte % 26 would bias the first 22 letters, which matters at this alphabet size. Parsing discards case and separators (space, dash, underscore, dot, colon, slash) then requires exactly ten letters, so xxx-yyyy-zzz, XXXX YYYY ZZ, and xxxyyyzzz all resolve to one canonical value. InviteLink holds exactly one target as a closed enum (InviteTarget::Room | Session) with a matching kind id, expiry, status (active/revoked/expired), and creator. Revoke is terminal. is_redeemable_at decides from the clock plus status, so expiry is derived rather than a stored flag and there is no sweep. Bounds: default 7 days, max 30.

Migration 0008_invite_link.sql. mahfel.invite_links with a UNIQUE code plus a regex CHECK for the canonical shape, target_kind/target_id, status CHECK, a CHECK that expires_at > created_at, and a CHECK that revoked and revoked_at are both present or both absent. target_id is deliberately NOT a foreign key: a polymorphic reference is impossible without a trigger, and link rows must outlive their target's operational lifetime. Resolution fails closed when the target is gone. Documented as deferred reaping.

Port and adapter. InviteStore with insert, get_by_code, get, list_for_target, revoke_if_active, and rotate. Insert returns Conflict on a duplicate code, which is what makes generation-with-retry possible. PostgresInviteStore implements it; rotate revokes the previous row and inserts the replacement in one transaction, so a reader sees exactly one live code and a duplicate code rolls the transaction back leaving the previous code live.

Use cases (crates/application/src/invite/). create (room or session target, session resolved to its room then manager-gated), list (manager-only, revoked and expired included), revoke (terminal, 409 on repeat), rotate (one implementation backing both rotate and regenerate; preserves the previous window when expiry_days is omitted so rotation cannot silently extend a short link), guard (authorizes a by-id link operation against the link's own target), and rate_limit. Bounded code retry lives in insert_with_unique_code and rotate; exhaustion is a deliberate error, not a loop.

Rate limiting. AttemptLimiter counts a fixed window per principal and per source address behind the CoordinationStore port, via a new increment_with_ttl implemented in Redis with an atomic INCR + EXPIRE pipeline. Redis because the window must be shared across replicas and is replaceable coordination state, not a business fact. Identifiers reach the key space only as a SHA-256 digest, so the namespace cannot be read as a directory of actors or addresses. A rejected attempt still consumes budget, and a limiter that cannot count refuses rather than silently allowing.

Transport timestamps reuse bins/api/src/http/routes/rfc3339.rs from TASK-123, so expires_at, created_at, and revoked_at are RFC 3339 on the wire.

REST. POST/GET /v1/rooms/{room_id}/invite-links, POST/GET /v1/sessions/{session_id}/invite-links, POST /v1/invite-links/{link_id}/revoke, POST /v1/invite-links/{link_id}/rotate. Registered in the route table, ApiDoc paths, components schemas, and REST_PATH_TEMPLATES; the OpenAPI coverage test passes. AppState gained invites and code_index; WsState gained code_index so a future socket redemption shares the same window.

Decisions.
- Session links are authorized through their room. The engine's model is that a session is managed by whoever manages its room, and there is no session-scoped capability context to invent here.
- Revoked rows are retained, not deleted, so a manager can see what was retired and when.
- Expiry is derived, never stored. The 'expired' status value exists in the vocabulary and CHECK but nothing writes it; it is reserved for an explicit future sweep.
- An unknown link id under a room the caller cannot see returns 404 (matching the shared guard, so room existence cannot be probed), while a caller who can see the room but lacks the manager role returns 403. Link ids are unguessable, so that distinction exposes nothing the caller could not already infer.

Validation. cargo fmt --check clean; clippy --workspace --all-targets -D warnings 0 errors; full workspace suite green (37 suites; domain 45, application 45, api 91). Postgres integration under MAHFEL_TESTCONTAINERS=1 passes for migrations (count now 8), invite, participants, participation, sessions, and service principals. just docs builds. sqlfluff reports only the pre-existing 0004_identity.sql LT02, left alone to preserve the SQLx checksum.

Scope note. Redemption itself is M4-14 (TASK-130): this task delivers the model, management, normalization, and the rate-limit/audit primitives that redemption calls. The AppState.code_index field carries an explicit #[expect(dead_code)] with a reason until M4-14 adds the first caller.
<!-- SECTION:NOTES:END -->
