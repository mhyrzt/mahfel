---
id: TASK-31
title: M4-09 Session placement scheduler
status: Done
assignee:
  - '@codex'
created_date: '2026-09-08 00:31'
updated_date: '2026-09-23 01:25'
labels:
  - backend
  - media
milestone: m-4
dependencies:
  - TASK-30
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 39000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Every participant in a session must reach the same capable media node. Implement atomic placement leases, a weighted capacity policy, sticky lookup for the lifetime of a session, explicit race and failure behavior, and a durable diagnostic record of placement decisions.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Concurrent joins to the same room select exactly one node through an atomic lease.
- [x] #2 Overloaded, draining, and dead nodes are excluded, and the weighted policy spreads load across healthy nodes.
- [x] #3 Sticky lookup returns the same node for the session lifetime, and a placement failure returns an explicit unavailable result with safe retry.
- [x] #4 Redis race tests prove that concurrent lease acquisition cannot double-place a session.
- [x] #5 Placement decisions (node, room, capacity, reason) are recorded durably and queryable by operators.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add a session-scoped placement contract and one immutable PostgreSQL decision row per session. 2. Order eligible media nodes by available room capacity and claim a node with an atomic Redis lease that rechecks heartbeat, drain, and room count. 3. Preflight both direct joins and lobby approvals, create a session-scoped internal SFU room on the winning node, then write participation. 4. Route RTC, moderation, cleanup, session end, and speaker observation through the assigned node while preserving browser room IDs. 5. Return unavailable on registry, lease, or assigned-node failure without moving a live session. 6. Document keys, diagnostics, lifecycle, and limits; verify Redis races, durable uniqueness, application policy, API behavior, formatting, lint, SQL, and docs.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Decision: PostgreSQL keeps one immutable session-to-node assignment; Redis uses a 20-second atomic lease and reservation. The API creates an SFU room keyed by session ID before admitting a participant, while browser room IDs stay unchanged. End cleanup deletes the assigned SFU room. A missing assigned heartbeat returns unavailable rather than moving an active session. Same-ID process restart and recovery negotiation remain M13-06 concerns.

Validation: API 129 tests passed; application session 26 and placement 3 tests passed; Redis Testcontainers race/drain/expiry test passed; PostgreSQL Testcontainers session/placement test passed; cargo fmt --check, clippy --all-targets -D warnings, SQLFluff on 0014, mkdocs strict build, and git diff --check passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented atomic weighted media placement with durable sticky session decisions, per-session SFU rooms, node-aware signaling/control/speaker routing, and explicit unavailable behavior. Verified Redis and PostgreSQL races plus API/application checks and documentation.
<!-- SECTION:FINAL_SUMMARY:END -->
