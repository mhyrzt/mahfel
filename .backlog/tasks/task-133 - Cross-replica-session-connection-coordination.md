---
id: TASK-133
title: Cross-replica session connection coordination
status: Done
assignee:
  - '@codex'
created_date: '2026-09-21 12:17'
updated_date: '2026-09-23 17:32'
labels: []
milestone: m-5
dependencies:
  - TASK-34
references:
  - docs/architecture/components.md
documentation:
  - docs/adr/0017-redis-pubsub-ephemeral.md
priority: medium
ordinal: 141000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: session participation builds on ConnectionRegistry, which is process-local. Session roster fan-out, per-connection generation ordering, and presence counting therefore only work within one API replica. On two replicas a session produces silent partial rosters (a join on replica B never reaches a participant on replica A), a stale cleanup cannot see a newer connection on another replica, and presence is counted once per replica. No error surfaces, so the failure is invisible.

This is not M1-06 work: TASK-7 is Done and delivered only the Redis coordination adapter (CoordinationStore, pub/sub via ADR 0017, TTLs, metrics). Connection state was never wired onto it. The mechanism to make connection state replica-safe is M5-03 (TASK-34), whose acceptance criterion already targets exactly this: clients on two API replicas receive committed state once.

Implementation scope: decide pub-sub versus streams for session/roster events (ADR 0017 defers Streams until at-least-once is required and warns pub-sub has no replay, so a replica that reconnects misses events permanently - a resync or snapshot story is required); route a cross-replica connection hub; make generation ordering replica-safe or explicitly scope it; derive presence across replicas; test two-replica roster consistency and reconnect resync under a dropped subscriber.

Out of scope: durable participation identity, uniqueness, and history, which are enforced by PostgreSQL (mahfel.session_participants, participants_one_active_per_session_actor_idx) and are already replica-safe.

Trigger: raised while implementing M4-02 (TASK-27); recorded in docs/architecture/components.md under the realtime gateway's 'Known limitations' and on ConnectionRegistry.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Clients on two API replicas in the same session observe the same roster after a join and a leave.
- [x] #2 A subscriber that drops and reconnects does not permanently miss roster state (resync or snapshot), given ADR 0017 pub/sub has no replay.
- [x] #3 Presence, meaning at least one live connection, is computed across replicas rather than once per replica.
- [x] #4 Generation ordering either holds across replicas or is explicitly documented as replica-scoped with the boundary named.
- [x] #5 Durable participation guarantees remain enforced by PostgreSQL and are unaffected by the connection hub.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add shared per-connection Redis leases and atomic session/actor presence transitions. 2. Serialize join and last-leave lifecycle across replicas while preserving PostgreSQL participant identity and intervals. 3. Fan out versioned roster snapshots to local WebSocket clients using Redis wakeups and periodic resync. 4. Keep local generation semantics explicit and fence stale remote cleanup by connection token. 5. Test two replicas, missed wakeups, multi-device leave, expiry, ordering, and durable participation; update contracts and architecture documentation.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented namespaced Redis per-connection leases with 45-second TTL, global actor/session PostgreSQL advisory locking, versioned session_roster snapshots, pub/sub wakeups plus two-second polling recovery, and replica-local generation documentation. Tests: two-registry delivery/dedupe, real two-replica Redis fanout without a subscriber, Redis stale cleanup/expiry, advisory-lock serialization, and existing participant uniqueness. Full backend test recipe passed (361 unit, 383 integration, doc tests); backend fmt/lint, docs, generated contracts, frontend typecheck and changed generator lint passed. Test recipe now bypasses the host proxy for loopback fixtures.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Session roster and presence now converge across API replicas; last-device leave uses a shared lease count under a PostgreSQL lifecycle lock. Missed pub/sub notices recover from Redis snapshots, with durable participant identity unchanged.
<!-- SECTION:FINAL_SUMMARY:END -->
