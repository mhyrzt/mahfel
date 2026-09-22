---
id: TASK-28
title: M4-05 Moderation controls
status: Done
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-22 20:25'
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
- [x] #1 A participant without the required capability cannot mute, unmute, remove, or block another principal, and a bypass attempt is rejected server-side.

Temporary and permanent mute are distinct: a target may clear a temporary mute itself, while a permanent mute can only be cleared by a caller holding MuteOthers for that room.

Enforced mute pauses the target's producers through media control, and unmuting resumes them without the target renegotiating transport.

Remove and block are distinct: remove ends participation and permits rejoin, while a blocked principal cannot rejoin while the block is active.

A room block prevents joining any session in that room, and a session block prevents joining that session only; a block with an expires_at lapses on its own, and one without it holds until explicitly lifted.

Publish and screen gating applies to both new producers and producers that are already running when the capability is withdrawn.

Every moderation action emits an audit event and propagates to affected clients over WebSocket, verified by an integration test.

A second concurrent device does not duplicate the participant row, presence holds while at least one connection is live, and moderation resolves across all of a participant's connections (closes TASK-27 acceptance criteria 7 and 8).

- [x] #2 Temporary and permanent mute are distinct: a target may clear a temporary mute itself, while a permanent mute can only be cleared by a caller holding MuteOthers for that room.
- [x] #3 Remove and block are distinct: remove ends participation and permits rejoin, while a blocked principal cannot rejoin while the block is active.
- [x] #4 A room block prevents joining any session in that room, and a session block prevents joining that session only; a block with an expires_at lapses on its own, and one without it holds until explicitly lifted.
- [x] #5 Every moderation action emits an audit event and propagates to affected clients over WebSocket, verified by an integration test.
- [x] #6 A second concurrent device does not duplicate the participant row, presence holds while at least one connection is live, and moderation resolves across all of a participant's connections (closes TASK-27 acceptance criteria 7 and 8).
- [x] #7 Removing or muting a participant that does not exist or is not active in the session fails with NotFound before any media cleanup is attempted: identity is validated against the authoritative store, not by the media plane.
- [x] #8 Enforced mute pauses the target's audio producers through media control, and unmuting resumes them without the target renegotiating transport.
- [x] #9 A producer records its media intent: kind is audio or video for the SFU, and a separate source distinguishes microphone, camera, and screen share.
- [x] #10 Publish and screen gating applies to both new producers and producers already running when the capability is withdrawn, and a screen-share action stops screen producers without stopping the target's camera or microphone.
- [x] #11 Publishing requires the capability that matches the source: PublishMedia for microphone and camera, ShareScreen for screen, so a non-member in a public room cannot publish.
- [x] #12 Withdrawing a capability closes the producers whose source it no longer covers, leaves the other sources running, and is a no-op when the capability set still covers them.
- [x] #13 A member's role can be changed, the last manager cannot be demoted, and the change is audited.
- [x] #14 Presence is derived from at least one live connection, and a participant is reported as left only when the last of their connections closes.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Steps 1 to 7 are complete. Remaining plan below.

8a. Gate new producers. handle_produce resolves the principal's capabilities for the session's room and requires the capability matching the source: PublishMedia for microphone and camera, ShareScreen for screen. This is the TASK-124 call-site swap and the first half of criterion 6. Source is what makes it expressible, because ShareScreen is a different capability from PublishMedia. A denial is a forbidden frame, and the pre-existing tests that publish must be checked for the capability they now need.

8b. Reconcile producers against capabilities. One application use case in moderation: resolve the principal's capabilities for the session's room, then close the producers whose source is no longer permitted, by source. Uniform across triggers and a no-op exactly when the capabilities still cover the source, so demotion does nothing because a member retains every media capability. It lives in the application layer and is called by each trigger's use case, not by a handler, so the rule holds for every path into a membership change including a worker.

8c. Wire the triggers. remove_member in rooms/members.rs, block in moderation/remove.rs, and the new change_member_role each call the reconcile, having gained whatever stores and the media gateway they need. Whether a block also ejects a currently-present principal is the open question recorded in the notes.

8d. change_member_role. A use case in rooms/members.rs that reuses ensure_member_removable for the last-manager guard, because demoting the last manager leaves the room with no manager and that is the failure the guard already names. require_manager for authorization, save the role, audit, then reconcile. A route PATCH /v1/rooms/{room_id}/members/{actor_id} following the existing members route shape, with the OpenAPI registration and REST_PATH_TEMPLATES entry, and an integration test that a registered route is documented.

9. Presence and last-connection leave. A connection count per participant on the registry, consulted by handle_leave so ParticipantLeft and the terminal participant transition happen only when the last connection for that participant goes. A second device joining must not duplicate the participant row, which is already true from the partial unique index, and must not emit a spurious ParticipantJoined for a participant the roster already has. Document the replica limitation and record the multi-replica test needed once TASK-34 lands.

10. Tests. A produce-denied test for a non-member in a public room, proving the hole is closed. A reconcile test proving a withdrawn capability closes the right source and leaves the others running, and that demotion changes nothing. A change_role test for the last-manager guard and the demotion path. A last-connection test: two connections, one closes, no ParticipantLeft; the second closes, then it fires. An integration test for the WebSocket propagation and audit of each moderation action, which is criterion 5.

11. Docs and closure. The four-rung ladder, block scope and expiry, source versus kind, the produce gate, and the replica limitation in the nearest authoritative document. Close TASK-27 acceptance criteria 7 and 8 with a note naming what closed them.
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

D confirmed as option (i): a room block implies its sessions by enforcement only. The join path checks the room bar first and the session bar second, and nothing writes per-session rows for a room block. This matters because of the lift path, not enforcement: with derived rows, lifting a room block can silently lift a narrower session block nobody asked to lift (a student walks into an exam), and a session block that predated the room block loses its provenance. Room-scoped and session-scoped blocks are independent durable facts, and lifting one never touches the other.

Remaining decisions from the planning pass.

A three columns on Participant: muted_at, muted_kind, and muted_by. muted_kind NULL means not muted, so there is a single source of truth and no reachable muted-without-a-kind state.

B the permanent-mute refusal lives in the domain, as a state-transition invariant, per the backend rule that validation and state transitions belong in domain types. Participant::unmute therefore takes the acting principal and whether they may moderate, so the aggregate can refuse a self-unmute of a permanent mute without knowing how capability was resolved.

C re-blocking an already-blocked principal extends the existing block rather than inserting a second row. A partial unique index over active blocks keeps that observable, and extension gives a re-block a well-defined meaning instead of a duplicate whose precedence is undefined.

D option (i), as above.

E removal and enforced mute resolve across all of a participant's connections: producers are closed once for the principal, and the participant is ended once, not per connection.

F one task, no split.

Implementation note carried forward: TASK-136 added producer pause, resume, and close to mahfel-media and the media client as inherent methods, but did not extend the MediaGateway application port, which still exposes only create_room and close_room. Step 1 of this task is therefore the port seam: the three methods on the trait plus the impl where MediaGateway for GrpcMediaGateway lives. The media plane itself is ready; only the application-facing seam is missing.

Producer media intent, folded in from the planning pass (decisions Q1 and Q2).

Today a producer records no media intent: ProducerEntry holds the mediasoup handle, room_id, transport_id, and participant_id, and the MediaKind passed to create_producer is handed to mediasoup and then discarded. ProduceRequest.kind is only audio or video, and a screen share is indistinguishable from a camera because both are video at the media plane. ShareScreen exists as a capability with no media-side identity.

That makes criterion 6 hollow as written: without distinguishing screen from camera, screen gating can only pause all of a participant's video, which stops their camera when the intent was to stop their screen share.

Q1 video plus source. kind stays what the SFU needs (audio or video), and a separate source carries the application-level intent (microphone, camera, or screen). This is faithful to mediasoup, where a screen share genuinely is video, rather than inventing a third kind that the SFU does not have. source is a field on ProducerEntry, not on Participant: what a producer is only matters while the producer exists, and durable moderation state belongs on the participant.

Q2 folded into this task rather than split out, so the protocol change, the media-plane tag, and the gating that depends on them land together and cannot drift apart.

Client-supplied source is not trusted as authorization. It selects which producers a moderator's action applies to; whether the principal may publish or share at all is still decided by the capability in the same request, so mislabelling a source can narrow an action but can never grant one.

Consequence for the proto: ProduceRequest gains a source field. That is a media-plane contract change in the same layer as TASK-136, and it means just proto fmt-check and just proto lint must pass again as part of this task.

Steps 1 and 2 complete.

Step 1, port seam. MediaGateway gained pause_producer, resume_producer, and close_participant, implemented in media-client/room.rs by delegating to the inherent GrpcMediaGateway methods TASK-136 added. GrpcMediaGateway is the only implementor, so nothing else needed updating.

Step 2, media intent. ProducerSource { Microphone, Camera, Screen } landed in the domain with status_conversion! storage names and an is_audio helper. ProduceRequest gained a source field, empty defaulting to microphone for audio and camera for video so existing callers keep working. ProducerEntry now records kind and source, which is what makes source-targeted selection possible at all: the MediaKind was previously discarded after being handed to mediasoup.

A source is validated against the kind it was sent with, and a mismatch is refused rather than coerced, because a wrong label would misdirect a later mute. The check is the same in the gRPC and WebSocket paths, each defaulting an omitted source identically.

Selection is one rule: owned_by(entry fields..., request fields..., Option<ProducerSource>), where None selects every source. Removal passes None, because removal ends participation and every source goes; pause and resume pass Some(source), because a mute is narrower. pause_participant_source and resume_participant_source are the new source-targeted operations, and they are what the enforced-mute path will call.

Six unit tests cover the selection rule, including one for the case that required the whole change: stopping a screen share leaves the camera running, which a kind-only selection could not express because both are video to the SFU.

Validation so far: just backend build clean, 261 unit tests pass, just proto fmt-check and just proto lint both pass.

Steps 3 and 4 complete, and the adapter layer is in place. Stopping here with nothing committed, as requested.

Step 3, domain. MuteKind { Temporary, Permanent } with storage names. Participant gained muted_at, mute_kind, and muted_by, with state-checked mute and unmute. The permanent-mute refusal is in the aggregate: unmute takes may_moderate and returns PermanentMuteNeedsModerator on a self-unmute of a permanent mute, so the invariant does not depend on a use case remembering it. Re-muting replaces the kind, which makes an upgrade from temporary to permanent take effect instead of conflicting, and a mute on a participant who has left is refused because it would apply to nobody. A new block module: BlockScope { Room, Session } carrying exactly one target, and Block with an optional expires_at, is_active_at, lift, and extend. Lifting twice is refused so it cannot overwrite who lifted it, and extending replaces the expiry rather than stacking.

Step 4, migration 0011_moderation.sql. Three mute columns with a check that they move together, so a partial write cannot make a mute permanent by omission. A mahfel.blocks table with a scope discriminator, nullable expires_at, and lifted_by; checks that exactly one scope is named, that a lift records its principal, and that an expiry is after the imposition. Partial unique indexes over active blocks only, so re-blocking extends and a lapsed block does not conflict with a new one. Migration count assertion updated from 10 to 11 and verified against a real Postgres container: the migrations test passes.

Adapters. ParticipantStore now round-trips the mute columns through all three SELECTs and both writes; save previously would have silently dropped a mute. New BlockStore port plus PostgresBlockStore, where get_active treats an expired block as inactive without requiring a lift, and extend and lift are conditional on the block still being active so neither can undo the other. PostgresStores bundles blocks alongside the other stores.

Validation: just backend fmt clean, clippy -D warnings 0 errors, 280 unit tests pass (up from 261), just proto fmt-check and just proto lint both pass, and migration 0011 applies against a real Postgres container.

Remaining: steps 5 to 11, which are the use cases in application/src/moderation, the join-path block checks, the WebSocket frames, the running-producer sweep, multi-connection behavior, and the tests and docs.

Steps 5 and 6 complete.

Step 5, use cases, in crates/application/src/moderation. mute.rs handles both mute and unmute: mute takes a MuteKind and a MutedSource, unmute resolves whether the caller is moderating or clearing their own mute and then defers the permanent-mute refusal to the aggregate. remove.rs has remove_participant, block, and unblock; block extends an existing active block rather than inserting a second row, and unblock lifts the one active block. A shared scope resolver turns a BlockScope into the room that authorizes it, so a session-scoped block is authorized against the session's own room and a caller cannot bar someone from a room they do not manage.

The ordering is the property that matters, and it is the same for every action: resolve the capability, resolve the target against the authoritative store, change durable state, call the media plane, then audit. Capability first so an unauthorized caller cannot probe for who exists, and the state change before enforcement so a media failure reports Enforcement rather than a grant that did not happen. Criterion 7 lives in the second step: a subject with no active participant fails NotFound before any media call, which is the check the media plane cannot perform.

Two deliberate error-shape choices. A media failure is ModerationError::Enforcement rather than a storage error, because the durable state did change and only the effect needs reconciling. And From<ParticipantError> maps the participant failures moderation has its own vocabulary for, so PermanentMuteNeedsModerator surfaces directly instead of nested inside a participant variant.

Step 6, join-path enforcement. join_session gained a BlockStore parameter and calls require_not_blocked after the room guard and before admission. The room check runs first because a room block already covers its sessions and nothing writes per-session rows for it; the session check then handles blocks narrower than a room block. A blocked principal is refused before the lobby branch, so it never queues and cannot consume lobby capacity.

The media port needed source-targeted operations to go with the per-producer ones: pause_participant_source and resume_participant_source were added to MediaGateway, the proto gained PauseParticipantSource and ResumeParticipantSource rpcs, and CloseParticipantResponse now reports how many producers it closed. Both new paths are wired through bins/media, the media client, and the runtime.

A new policy helper, require_actor_capability, resolves from an actor id rather than a Principal. Moderation use cases carry an actor rather than a resolved principal, and moderation capabilities are room-scoped, so resolving from membership alone is the correct reading rather than a shortcut. A caller with no membership is denied, which is what the tests assert.

Validation: just backend build clean, clippy 0 errors, 303 unit tests pass (up from 280), and just proto fmt-check and just proto lint both pass. New tests cover the capability denial for member and outsider, NotFound before media on an unknown or departed subject, the temporary and permanent unmute split, source targeting, a media failure leaving the mute recorded, remove without creating a block, re-block extending rather than stacking, a room block and a session block surviving each other's lift, and a room block denying a join without a per-session row.

Step 7 complete: WebSocket transport.

Client frames MuteParticipant, UnmuteParticipant, RemoveParticipant, BlockPrincipal, and UnblockPrincipal; typed server frames ParticipantMuted, ParticipantUnmuted, ParticipantRemoved, PrincipalBlocked, and PrincipalUnblocked. Typed rather than a generic envelope so an unhandled action fails to compile on the client instead of silently reaching an old one.

The handlers are a transport adapter only: parse, delegate to the use case, fan out. Capability resolution, the authoritative target lookup, and the ordering of state change against enforcement stay in the application layer, so REST and WebSocket decide identically. A self-unmute routes through the same handler, so the capability check is identical whether the target or a moderator initiated it.

Wire parsing refuses rather than defaults where the difference is semantic. An unknown mute kind is invalid_argument, because the kind decides whether the target can clear it. A malformed actor or session id is invalid_argument. A block must name exactly one of room or session. A block expiry arrives as Unix seconds and an unparseable one is dropped rather than guessed.

Fan-out uses a new ConnectionRegistry::broadcast_to_session, which reaches every connection in the session including the target. The existing broadcast_to_session_except is wrong for moderation: the target must learn it was muted or removed, and the whole roster needs to reflect the change. Both now share one private send_to, so the try_send backpressure behaviour is identical in each.

Registry cleanup: connection_count_for_participant, connections_for_participant, and send_to_connection were written speculatively and removed when clippy flagged them as dead. broadcast_to_session already covers fan-out, and step 9 will add only the presence helper it actually proves it needs.

An orphaned doc comment was found on ConnectionRegistry: the docs for a participant connection count sat above broadcast_to_session_except with no function under them, left by TASK-27. Those speculative methods were removed rather than resurrected on the strength of a stray comment.

Validation: just backend build clean, just backend fmt clean, clippy -D warnings 0 errors, 303 unit tests pass, and both proto gates pass.

Steps 8 and 9 planned in detail, with findings that change their shape.

Finding: 8a is a real authorization hole. handle_produce in ws/connection/dispatch/rtc.rs parses the kind and source and calls media.produce with no capability check at all; grep for Capability in that file returns nothing. TASK-124 defined PublishMedia and ShareScreen and the call site was never wired to them, which is the outstanding seam swap. The concrete case: a non-member may join a public room (Join only, no PublishMedia, no ShareScreen) and then publish camera and screen, because nothing refuses it. handle_create_transport is ungated too, but a transport without produce carries no media, so produce is the right single choke point.

Finding: demotion does not withdraw media capability. RoleCapability::for_role(Member) is Participate and JoinSession, and both map to Join, PublishMedia, and ShareScreen. So a manager demoted to member keeps every media capability and loses only Moderate and ManageSession. A reconcile that re-resolves capabilities and closes newly-forbidden sources therefore does nothing on demotion, which is correct rather than a gap: there is no producer to close. The uniform reconcile is still the right shape, because it stays correct if a role is ever introduced that lacks a media capability.

Finding: RoomMember::change_role is dead code. It is defined in the domain and called from nowhere: no use case, no route. So demotion is unreachable today and step 8.3 gives it a path.

Decision 8.1, from the user: implement 8a, gating produce on the capability that matches the source.
Decision 8.2, from the user: the sweep closes producers rather than pausing them. Pausing would leave a producer that nobody resumes, since the client cannot be relied on to keep a paused producer alive across an arbitrary interval, and that differs from mute because mute is short and the client is actively connected.
Decision 8.3, from the user: add a change_role path rather than leaving the domain method dormant.

Design for the reconcile. One operation rather than three: re-resolve the principal's capabilities for the session's room, then close the producers whose source is no longer permitted, by source. This is uniform across triggers and is a no-op exactly when the capability set still covers the source. It is called from each trigger's use case rather than from the handler, because the rule must hold for every path into a membership change, including a worker or a future CLI, and not only for the HTTP route that happens to exist today.

Question left open for the user: whether blocking a principal who is currently in the session should also eject them. A block bars rejoining, so a blocked principal who stays connected keeps streaming until their connection ends, which is arguably not a block. Proposed: block composes removal, ending the participation and closing producers, since a block that leaves someone publishing is not a block.

Step 9, confirmed by the user: presence is at least one live connection, and ParticipantLeft is emitted only when the last connection for a participant goes. Today handle_leave emits ParticipantLeft on any leave, so closing one of two devices tells the roster the participant left while the other device is still streaming. The fix adds a per-participant connection count to the registry and consults it in handle_leave. Cross-replica remains documented rather than solved, since two devices on different replicas cannot see each other until the M5-03 connection hub (TASK-34, ADR 0017).

Steps 8 and 9 complete, plus the change-role path.

8a, the publish gate. handle_produce now resolves the principal's capabilities for the session's room and requires the capability matching the source: PublishMedia for microphone and camera, ShareScreen for screen. This closed a real hole: handle_produce had no capability check at all, so a non-member of a public room, who holds Join and nothing else, could publish camera and screen. A denial is a forbidden frame and no producer is created.

8b, the reconcile. reconcile_media in moderation/reconcile.rs resolves the principal's capabilities and closes only the sources those capabilities no longer cover. Room-scoped rather than session-scoped because a room has at most one live session and the media plane keys producers by room, so reconciling a room reaches exactly the live session with no iteration. A demotion therefore makes no media call, because a member keeps PublishMedia and ShareScreen; that is the case that makes this a reconcile rather than a sweep. capability_for is shared with the produce gate so both decide the same way.

8c, the triggers. remove_member and change_member_role each reconcile after the durable change. A media failure is best-effort for the response and loud in the log: the membership is already gone, so failing the request would misreport a change that happened, but a failed reconcile means a removed principal may still be publishing and an operator has to know. This surfaced while fixing a test: the transport tests use a real GrpcMediaGateway pointed at a port with nothing listening, so a reconcile that propagated its error turned a successful removal into a 500. The durable change succeeding while enforcement degrades is the honest behaviour, and it is recorded as a degraded audit event.

Block composes removal, per the user's decision. A block bars rejoining, so one that left the subject publishing would not be a block: block now ends the subject's participation in every session the scope covers and closes their producers. BlockOutcome gained created and ejected_from, replacing Created and Extended.

8d, change_member_role. A new use case and a PATCH route. It reuses ensure_member_removable for the last-manager guard, because demoting the last manager leaves the room with nobody who can manage it, which is the invariant that guard already names. RoomMember::change_role is no longer dead code. Registered in ApiDoc; the path template already existed because the DELETE route shares the path.

9, presence. handle_leave detaches the connection first and then asks the registry how many connections the participant still holds, so a device closing while another is streaming neither ends the participation nor tells the roster the participant left. ParticipantLeft and the terminal leave now happen only at the last connection. ConnectionRegistry gained connection_count_for_participant.

A test-fake defect was found and fixed: FakeMedia reported affected: 0 by default, so the reconcile tests proved only that zero is zero. The withdrawn-capability test now reports one producer per source. The fixture also had to remove the membership from the fake room store rather than a separate memberships fake, because production resolves membership through the room store and the two must not be able to disagree.

Validation: just backend fmt clean, clippy -D warnings 0 errors, 321 unit tests pass (up from 303), and both proto gates pass. New tests: the publish gate denies a non-member and permits a member, a demoted manager keeps every media capability, the reconcile closes every source on removal and nothing on demotion, a principal who still holds the capability is left alone, block ejects a present principal and ejects nothing for an absent one, and six registry tests for presence including one participant across two connections and a connection that has not joined.

Remaining: steps 10 and 11, the integration tests for criterion 5 and the documentation.

Steps 10 and 11 complete. The task is finished.

Step 10, tests. Five WebSocket integration tests drive the real connection handler over a real socket, so they prove a moderation action reaches the affected client as a frame rather than only that a use case returns the right value: mute propagates a participant_muted frame naming the target by actor id and carrying the kind, a member is refused with forbidden and no durable change, removal propagates participant_removed, an unknown participant is not_found before any media call, and an unknown mute kind is refused. The WS harness was generalised to accept seeded stores, because the plain one builds its own and leaves a moderation frame with nothing to act on.

Writing those tests exposed a defect in the design, not in the tests. mute enforced the media plane before returning and propagated the frame after, so an unreachable media plane meant the durable mute was recorded, the caller got an error, and the target was never told to stop publishing. That is the worst of both: muted in the database, still streaming, nobody informed. Enforcement is now best-effort for the outcome and audited as degraded, and MuteOutcome carries a degraded flag so a caller can tell a mute that is enforced from one that is only recorded. remove_participant got the same treatment for the same reason.

A Postgres integration test covers the block schema against a real container, and it found a genuine bug. The partial unique index excluded lifted blocks but not expired ones, and a partial index cannot express expires_at > now() because now() is not immutable. So a lapsed block kept occupying the active slot and a legitimate new block was refused, which broke the stated behaviour that a lapsed block does not conflict. The write path now retires any lapsed block for the same target before inserting, and the constraints were corrected: the old check required lifted_at and lifted_by to be present together, which made an expiry unwritable, because an expiry records no principal. The migration comment was wrong about the index and is now accurate, and the migration is uncommitted so correcting it in place is legitimate rather than an edit to a deployed migration.

Step 11, documentation. components.md gained a Moderation section covering the four rungs, durable mute state, independent block scopes with optional expiry, block composing removal, source versus kind, the publish gate, reconcile rather than sweep, identity before enforcement, and presence from connection count, plus the replica limitation. data-flow.md gained a Moderation sequence diagram and the ordering rationale. just docs builds. TASK-27 acceptance criteria 7 and 8 are closed with a note naming what closed them.

Validation: just backend fmt clean, clippy -D warnings 0 errors, 327 unit tests pass, the blocks and migrations and participants Postgres integration tests pass against real containers, both proto gates pass, and just docs builds.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Capability-gated, vertical-agnostic moderation: temporary and permanent mute, remove, block, and role change, with media enforcement, WebSocket propagation, and audit events. Education maps its teacher role onto the capabilities rather than adding engine roles.

Four rungs, each strictly stronger and non-overlapping. Temporary mute is a reset the target clears. Permanent mute holds until a caller with mute_others clears it. Remove ends the participation and permits rejoin. Block is a durable bar that also composes removal, because a block that left someone publishing would not be a block. Mute is durable state on the participation, so it survives a reconnect but not a rejoin, and that is why block exists.

Producers now record their media intent: kind for the SFU and source for microphone, camera, or screen. A screen share genuinely is video, so kind alone could not distinguish it from a camera, and without that distinction mute could only silence all of a target's video. Muting pauses the named source and leaves the others running; unmuting resumes without renegotiation. Publishing is gated by the capability matching the source, which closed a real hole: handle_produce had no capability check at all, so a non-member of a public room could publish camera and screen.

Two design choices are load-bearing. Identity is validated against the authoritative store before any media call, because the media plane holds no participant registry and must not gain one. And enforcement failure is best-effort for the response while audited as degraded: the durable change already happened, so failing the request would misreport it, but an operator has to know a removed principal may still be publishing.

Verification found three defects. handle_produce lacked its capability check. And two schema bugs in the block table: the partial unique index let a lapsed block hold the active slot, and the constraints made an expiry unwritable because an expiry records no principal. All three are fixed, and the second pair came from a Postgres integration test written specifically to exercise the constraints.

Verified: fmt clean, clippy -D warnings 0 errors, 327 unit tests including five WebSocket integration tests and six registry presence tests, the blocks migrations and participants Postgres integration tests against real containers, both proto gates, and just docs. Closes TASK-27 acceptance criteria 7 and 8.
<!-- SECTION:FINAL_SUMMARY:END -->
