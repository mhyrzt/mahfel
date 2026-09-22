---
id: TASK-28
title: M4-05 Moderation controls
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-22 14:24'
labels:
  - backend
  - api
milestone: m-4
dependencies:
  - TASK-136
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 36000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Capability-gated moderation that works for any vertical: temporary and permanent mute, remove, block, publish and screen gating, and propagation of moderation state over WebSocket, with audit events. An education deployment maps its teacher role onto these capabilities rather than adding new engine roles.

Implementation spans four rungs, each strictly stronger than the last, with no overlap:

1. Temporary mute: a moderator mutes now, the target's own unmute clears it.
2. Permanent mute: only a caller holding MuteOthers for that room clears it. The target's unmute is refused.
3. Remove: ends the target's participation now. They may rejoin.
4. Block: a durable bar on rejoining, while active.

Mute state lives on the durable Participant (a muted_at with kind and actor), not only in the media plane, so a reconnect reuses the same row and the mute holds. Mute is attached to a participation, not to a person: leave and remove are terminal, so a rejoin creates a new participant row and clears the mute. That is deliberate, and it is why rung 4 exists: a caller who needs a mute that survives rejoin wants a block, not a stronger mute.

Blocks exist at room level and at session level. A room block implies every session in that room, so the session list is for cases narrower than a room block. A block carries an optional expires_at: NULL means until explicitly lifted (the education case), a timestamp means it lapses on its own (the one-off meeting case).

Enforcement reaches the media plane through the MediaGateway port, which depends on TASK-136 implementing producer pause, resume, and close. Enforcement pauses rather than destroys where the target may be unmuted, so a resume needs no renegotiation.

This task also closes TASK-27 acceptance criteria 7 and 8: presence is derived from at least one live connection, and moderation resolves a participant unambiguously across all of its connections.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A participant without the required capability cannot mute, unmute, remove, or block another principal, and a bypass attempt is rejected server-side.

Temporary and permanent mute are distinct: a target may clear a temporary mute itself, while a permanent mute can only be cleared by a caller holding MuteOthers for that room.

Enforced mute pauses the target's producers through media control, and unmuting resumes them without the target renegotiating transport.

Remove and block are distinct: remove ends participation and permits rejoin, while a blocked principal cannot rejoin while the block is active.

A room block prevents joining any session in that room, and a session block prevents joining that session only; a block with an expires_at lapses on its own, and one without it holds until explicitly lifted.

Publish and screen gating applies to both new producers and producers that are already running when the capability is withdrawn.

Every moderation action emits an audit event and propagates to affected clients over WebSocket, verified by an integration test.

A second concurrent device does not duplicate the participant row, presence holds while at least one connection is live, and moderation resolves across all of a participant's connections (closes TASK-27 acceptance criteria 7 and 8).

- [ ] #2 Temporary and permanent mute are distinct: a target may clear a temporary mute itself, while a permanent mute can only be cleared by a caller holding MuteOthers for that room.
- [ ] #3 Enforced mute pauses the target's producers through media control, and unmuting resumes them without the target renegotiating transport.
- [ ] #4 Remove and block are distinct: remove ends participation and permits rejoin, while a blocked principal cannot rejoin while the block is active.
- [ ] #5 A room block prevents joining any session in that room, and a session block prevents joining that session only; a block with an expires_at lapses on its own, and one without it holds until explicitly lifted.
- [ ] #6 Publish and screen gating applies to both new producers and producers that are already running when the capability is withdrawn.
- [ ] #7 Every moderation action emits an audit event and propagates to affected clients over WebSocket, verified by an integration test.
- [ ] #8 A second concurrent device does not duplicate the participant row, presence holds while at least one connection is live, and moderation resolves across all of a participant's connections (closes TASK-27 acceptance criteria 7 and 8).
- [ ] #9 Removing or muting a participant that does not exist or is not active in the session fails with NotFound before any media cleanup is attempted: identity is validated against the authoritative store, not by the media plane.
<!-- AC:END -->



## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Order is riskiest first: the media-plane control is the only genuinely unknown part, and everything else is a well-trodden path in this codebase.

1. Media-plane control (dependency TASK-136). Consume PauseProducer, ResumeProducer, and CloseParticipant through the MediaGateway port in crates/application/src/ports/media_gateway.rs, implemented by GrpcMediaGateway in crates/media-client. The port today exposes only create_room and close_room, so this adds the three producer methods. No proto change happens in this task: TASK-136 owns proto/.

2. Domain (crates/domain/src/participant.rs and a new block module). MuteKind { Temporary, Permanent }, mute state on Participant (muted_at, muted_kind, muted_by) with state-checked mute and unmute, where a permanent mute refuses a self-unmute. Block as its own aggregate with a scope (room or session), an optional expires_at, and is_active_at(now). Unit tests for storage names, the self-unmute refusal, and expiry evaluation.

3. Migration 0011_moderation.sql. Mute columns on mahfel.session_participants. A mahfel.blocks table with a scope discriminator and expires_at NULL. Partial unique indexes over active blocks only, so a lapsed block does not conflict with a new one, following the pattern already used for participants and admission requests. The migration count assertion in crates/postgres/tests/migrations.rs goes from 10 to 11.

4. Ports and adapters. ParticipantStore persists mute state. A new BlockStore with get_active, insert, lift, and list_for_room or list_for_session. Postgres adapters for both, returning BoxFuture per ADR 0023, with unique violation mapped to StoreError::Conflict.

5. Application use cases in crates/application/src/moderation/. mute.rs (kind-aware, resolving whether the caller is moderating or unmuting themselves), remove.rs (terminal via Participant::remove, then close_participant), block.rs and unblock.rs, and publish.rs and screen.rs for the capability gate reused by the produce path. Each action checks the capability first, then changes state, then calls the media plane, then audits. Authorization is deny-by-default and reuses the existing resolver rather than adding a check.

6. Join-path enforcement. Block checks in crates/application/src/sessions/join.rs at the two points where authorization already runs: a room block denies, and a session block denies. A room block subsumes its sessions, so the session check does not duplicate room blocks.

7. WebSocket transport. New ClientMessage variants MuteParticipant, UnmuteParticipant, RemoveParticipant, BlockPrincipal, and UnblockPrincipal. New typed ServerMessage variants ParticipantMuted { muted, kind }, ParticipantUnmuted, ParticipantRemoved { reason }, PrincipalBlocked, and PrincipalUnblocked, matching the existing typed-variant protocol rather than a generic envelope. Fan out to the affected connections. A self-unmute routes through the same handler, so the capability check is identical whether the target or a moderator initiates it.

8. Publish and screen gating on existing producers (AC 6). Where a capability change happens, enumerate the principal's active producers through the media plane and pause them. Reuse the participant lookup that TASK-136 makes possible rather than adding a producer index.

9. Multi-connection behavior (AC 8). Registry presence is at least one live connection. Moderation resolves across all of a participant's connections. This is replica-local and cross-replica ordering stays deferred to the connection hub (TASK-34, M5-03, ADR 0017); document that limitation, and add an implementation note naming the multi-replica test that will be needed once the hub lands.

10. Tests and docs. Integration test proving an audit event plus WebSocket propagation for each action (AC 7). Postgres integration test for block uniqueness, expiry, and the room-to-sessions subsumption. Docs: the four-rung ladder, block scope and expiry, and the replica limitation in the nearest authoritative document. Close TASK-27 acceptance criteria 7 and 8 with a note explaining what closed them.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Decisions locked before implementation.

D1 mute state is durable on Participant, not media-plane only. A reconnect reuses the same (session, actor) row, so the mute holds across a transport change; a media-only pause would not survive the producer being recreated. Postgres stays authoritative and the media producer stays a volatile resource.

D2 two mutes. Temporary: a moderator mutes now, the target's own unmute clears it. Permanent: only a caller holding MuteOthers for that room clears it, and the target's unmute is refused. Four rungs in total (temporary mute, permanent mute, remove, block), each strictly stronger, with no overlap.

D3 blocks exist at room level and at session level. A room block implies every session in that room, so the session list covers cases narrower than a room block (excluded from one session but still in the room).

D4 remove and block are orthogonal. Remove ends participation and allows rejoin; block is the durable bar. Collapsing them would erase acceptance criterion 4.

D5 a capability withdrawal sweeps producers that are already running. Produce-time-only checks are explicitly insufficient, because a demoted principal keeps broadcasting a producer that was authorized before the change.

D6 replica-local for now, documented. The registry sees only this replica's connections, and cross-replica ordering remains with TASK-34 (M5-03, ADR 0017). An implementation note below names the multi-replica test needed later.

D7 typed WebSocket frames, matching the existing protocol, so an unhandled moderation action fails to compile on the client rather than silently reaching old clients.

D8 this task closes TASK-27 acceptance criteria 7 and 8.

D9 pause for mute, close for remove. Pause keeps mute state and media state in 1:1 correspondence: a paused producer resumes without renegotiation, and the durable mute flag never points at a destroyed producer. Closing is correct for remove, which is terminal.

Q2 mute does not survive a rejoin. Leave and remove are terminal, so a rejoin creates a new participant row and clears the mute. Deliberate: a caller who needs a mute that survives rejoin wants a block, which is rung 4.

Q3 blocks carry an optional expires_at. NULL means until explicitly lifted, which is the education case; a timestamp means it lapses on its own, which is the one-off meeting case. A single nullable column rather than a permanent flag, because two ways to express permanence drift apart.

Discovered during planning: CloseParticipant is declared in the proto and mapped in bins/media/src/service.rs, but returns Status::unimplemented. TASK-20 listed cleanup on CloseParticipant as part of its plan while only the DeleteRoom path was wired. Enforcement through media control was therefore impossible before TASK-136, which is why the media-plane work is a separate task with its own proto and validation gate.
<!-- SECTION:NOTES:END -->
