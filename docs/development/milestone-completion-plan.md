# M3–M5 completion and existing bug reports

Planning snapshot: 2026-10-07. Source baseline: `7938ae8` plus the existing working-tree documentation changes.
This is an execution proposal, not evidence that the fixes or browser checks have passed.
GitLab remains the source of issue state; this document supplies ordering and acceptance gates.

Implementation and local acceptance evidence are recorded in the
[completion report](milestone-completion-report.md).

## Scope and current evidence

Complete the four remaining milestone issues: [RTC client #71](https://hamgit.ir/hamdars/mahfel/-/issues/71),
[moderation/admission UI #175](https://hamgit.ir/hamdars/mahfel/-/issues/175),
[session lifecycle UI #176](https://hamgit.ir/hamdars/mahfel/-/issues/176), and
[invite UI #179](https://hamgit.ir/hamdars/mahfel/-/issues/179).
Also triage and resolve the 40 pre-migration reports, issues #1–#40. All 40 were open when read.
M5 has no open milestone task, but its completed-task count does not settle the separate chat reports.

Planning inspection confirmed these source patterns; it did not run a runtime reproduction:

| Area | Observation | Planning consequence |
| --- | --- | --- |
| RTC admission | `backend/bins/api/src/http/ws/connection/dispatch/mod.rs` now checks joined participation and matching room before RTC dispatch. | The original #1 description is partly stale; audit ownership and regression coverage before declaring it resolved. |
| Multi-device teardown | `backend/bins/api/src/http/ws/connection/session.rs` uses lifecycle locks and Redis presence when deciding last-device cleanup. | Revalidate #2–#6 against current code; do not reimplement old fixes from descriptions alone. |
| Session editing | `frontend/features/rooms/session-settings-dialog.tsx` already PATCHes title, start, and duration; `tests/session-settings-dialog.test.tsx` exercises the dialog. | #176's claim that editing is absent is stale. Verify persistence, permissions, clear/reset values, and live behavior. Existing tests were read, not run. |
| Membership updates | `change_member_role` calls insert-only `add_member` and suppresses conflicts; the PostgreSQL insert uses `ON CONFLICT DO NOTHING`. | #7 remains a source-supported persistence concern. |
| Admission scope and atomicity | Pending lookup omits the room discriminator for room requests; `admit` grants state before `decide_if_pending`. | #8/#9 need store isolation and transactional concurrency tests. |
| Media cleanup/recovery | Send setup can commit resources after awaits without an unmount generation check; transport failure reuses original ICE parameters. | #37 and the ICE part of #40 directly block robust #71 sign-off. |
| Capability policy | `require_actor_capability` reconstructs a principal with an empty roles list. | #13 needs a shared verified-principal policy and a capability/action agreement matrix. |
| Browser security | The BFF body check is header-only; forwarding filters are incomplete; logout accepts protocol-relative paths. | #33–#36 require boundary tests and focused fixes before browser acceptance. |
| Media/worker reporting | Node participants are hardcoded to zero; worker recovery remains inside relay; publication-result boolean is discarded. | #30/#29/#32 remain source-supported concerns. |

Other reports are unverified until phase 0. Source inspection alone is insufficient to close an issue.

## Execution order

### Phase 0 — Reproduce, reconcile, and map the existing work

Read each issue's latest discussion and compare with current source and tests. Classify it as:
reproduced, source-supported pending reproduction, already fixed pending verification, or requiring
an acceptance/contract clarification. Record reproduction steps, affected component, test scenario,
and closure evidence on its original GitLab issue during implementation. Keep issue IDs and prior
history; do not create duplicate bug issues or silently move unrelated work between milestones.

Start with #1–#6 and #176 because newer source visibly differs from the reports. Run the existing
focused regression tests before adding tests or changing implementations. Check all four remaining
milestone checklists against the actual browser flow.

Exit: each of the 44 issues has a disposition and an explicit next action; already-fixed issues
have passing focused evidence before closure. No milestone is closed merely because its count is 100%.

### Phase 1 — Authorization, durable decisions, and browser boundaries

Issues: #1, #7–#9, #12–#14, #17–#18, #31, #33–#36.

- Bind RTC operations to admitted session, actor, and owned resources, including post-removal access.
- Add an explicit persisted membership update while preserving insert/conflict semantics for creation.
- Scope admission lookups/counts to the correct room/session. Commit the admission decision and its
  participant or membership grant in one PostgreSQL transaction through an application-owned port.
  A rejected or failed concurrent decision must leave no grant; compensation after a race is insufficient.
- Make capability introspection and action enforcement use the same verified-principal policy.
  Check roster privacy for blocked, waiting, non-member, member, manager, moderator, and admin callers.
- Reject overflowing pagination inputs with a deliberate validation error.
- Bound main and collaboration sockets, idle lifetime, inbound rates, and amplification. Preserve
  cleanup handles when closing overloaded connections. Remove or restrict production debug broadcast.
- Authenticate private media control using an existing-stack mechanism and document configuration
  and rotation. Keep the service private; authentication does not justify public exposure.
- Enforce actual streamed BFF body bytes, including missing or dishonest length. Filter hop-by-hop,
  `Connection`-nominated, and client forwarding headers. Validate the final upstream origin and
  allowlisted normalized path. Restrict logout destinations to the configured origin.

Exit: integration tests prove room isolation, persisted promotions/demotions, admission-race atomicity,
capability agreement, roster denial, RTC ownership, connection/rate bounds, and private-control denial.
Frontend boundary tests cover body overflow, encoded traversal, forwarding headers, and logout redirects.

### Phase 2 — RTC ownership, teardown, and reconnection; complete #71

Issues: #2–#6, #23–#27, #30, #37, #39–#40; milestone issue #71.

- Verify last-device presence, repeated/sequential join policy, generation-state bounds, and
  connection-owned teardown on disconnect, leave, removal, and session end.
- Release media map guards before awaited operations; verify races cannot resurrect closed resources.
  Prune consumers when producers close. Repeated create/disconnect cycles return counts to baseline.
- Add generation/cancellation checks after async browser media setup; stop late-resolving tracks,
  producers, and transports. Keep signaling connect single-flight and handshakes time-bounded.
- Apply bounded deadlines to all media control calls. Do not retry non-idempotent creation unless
  server-side deduplication is actually provided. Prefer disabling unsafe create retries for the
  first fix; add compatible idempotency fields only if needed and update producers/consumers together.
- Replace stale-parameter ICE restart. Prefer a bounded rejoin flow using existing contracts; if true
  transport restart is necessary, request fresh server ICE parameters and verify the whole contract.
- Tolerate bounded transient presence failures within the lease lifetime without weakening admission
  or authentication. Any placement cache must expire/invalidate on reassignment/drain and must never
  cache authorization grants as permanently valid.
- Define accurate participant-stat semantics and verify them against real active resources. Remove
  duplicate large presentation-byte state and release object URLs on replacement/unmount.

Exit: two independent authenticated browser contexts publish and consume microphone/camera/screen
media over real signaling and SFU paths. Cover Chromium and Firefox live publishing/consumption,
permission denial and retry, screen-share ended, network interruption/rejoin, two devices for one actor,
and repeated teardown. Browser replay in the existing recording test does not prove Firefox live RTC.

### Phase 3 — Moderation, session lifecycle, and invitations; complete M4

Issues: #10–#11, #15, #20; milestone issues #175, #176, #179. Depends on phases 1 and 2.

- Wire capability changes, enforced mute, publication/screen restrictions, removal/block, and
  admission decisions through authoritative server state and clear participant feedback.
- Deliver/reconcile moderation and roster changes across two API replicas. Redis pub/sub is an
  ephemeral notification path; PostgreSQL policy is truth. Missed notifications must recover through
  authoritative reconciliation, and revoked access must never depend solely on message delivery.
- Validate scheduled/live/ended/cancelled transitions, title/duration edits and clears, advisory
  duration, stable overlapping active speakers, and durable participation summaries.
- Bound session listing, batch roster lookups with caller-specific authorization intact, and bound
  avatar work. Make participation aggregation linear or near-linear and preserve interval semantics.
  Update API contracts and all existing pagination callers together without silently breaking them.
- Verify room and session invite management, full copied URLs, rotate/revoke/expiry, authenticated
  redemption, approval/rejection, and correct room-versus-session grants.
- Correct #179's criteria explicitly: authorized managers can see link status; unauthenticated or
  ordinary redeemers must not distinguish expired, revoked, and unknown codes. ADR 0021 requires
  non-enumerating errors. Rejected pending requests can still show their own authenticated status.

Exit: host and attendee on separate API replicas see correct moderation, admission, lifecycle,
speaker and participation behavior. Include a third unauthorized/blocked actor, two rooms, competing
admit/reject requests, and Persian RTL plus English, keyboard, and responsive UI checks.

### Phase 4 — Chat correctness, bounded collaboration state, and M5 sign-off

Issues: #16, #19, #21–#22, #38. #17/#18/#39 are handled in earlier phases.

- Route slow-consumer eviction through normal socket cleanup, preserving the participation handle.
- Batch fanout work without treating one subscriber's permission as authority for all subscribers.
  Verify membership revocation and reconnect cursor behavior with durable event replay.
- Apply incoming chat events or bounded sequence deltas instead of fetching full history per event;
  test create/edit/delete/reaction events, duplicates, gaps, and session-scoped filtering.
- Give speaker/fanout maps explicit eviction lifecycles. Bound checkpoint retention transactionally
  while keeping a reconstructible CRDT state and required replay data.

Exit: multi-client, multi-replica chat survives reconnect and slow-consumer eviction; durable presence
is cleaned up and event cursors stay correct. Measure database query count and HTTP refetch count under
bursts against a captured baseline. Check CRDT convergence/reconstruction and retention bounds.
M5 has no remaining feature task; close it after its relevant regression/sign-off checks pass.

### Phase 5 — Worker reliability reports

Issues: #28, #29, #32. This work can ship separately from M3/M4/M5 acceptance after triage.

Isolate per-delivery persistence/ack failures from other concurrent work while preserving at-least-once
semantics and lease ownership. Run expired-claim recovery independently of broker connectivity.
Handle a lost publication lease explicitly and expose bounded-cardinality outcomes.

Exit: fault-injection tests cover a DB failure on one job with others in flight, broker outage and
expired-claim recovery, and lease loss after a publish confirmation. No duplicate durable effects.

## Complete issue coverage

The assignment below is a proposed execution group, not a changed GitLab milestone or claim of severity.
Phase 0 applies to every issue. Phase groups become focused changes after reproduction; do not make
one large implementation patch spanning all reports.

| Issue | Report | Execution group |
| --- | --- | --- |
| [#1](https://hamgit.ir/hamdars/mahfel/-/issues/1) | Deny RTC signaling before session admission and enforce resource ownership | Phase 1 |
| [#2](https://hamgit.ir/hamdars/mahfel/-/issues/2) | Keep participant active while another device remains connected | Phase 2 |
| [#3](https://hamgit.ir/hamdars/mahfel/-/issues/3) | Prevent sequential WebSocket joins from orphaning participation | Phase 2 |
| [#4](https://hamgit.ir/hamdars/mahfel/-/issues/4) | Bound or remove unused WebSocket generation state | Phase 2 |
| [#5](https://hamgit.ir/hamdars/mahfel/-/issues/5) | Clean up media transports and tracks when participants disconnect | Phase 2 |
| [#6](https://hamgit.ir/hamdars/mahfel/-/issues/6) | Release media map locks before awaiting mediasoup operations | Phase 2 |
| [#7](https://hamgit.ir/hamdars/mahfel/-/issues/7) | PATCH room member role change is silently not persisted | Phase 1 |
| [#8](https://hamgit.ir/hamdars/mahfel/-/issues/8) | Room-scoped admission lookups ignore room_id | Phase 1 |
| [#9](https://hamgit.ir/hamdars/mahfel/-/issues/9) | Admission grant happens before the conditional decision (reject race) | Phase 1 |
| [#10](https://hamgit.ir/hamdars/mahfel/-/issues/10) | Session list endpoint is unbounded | Phase 3 |
| [#11](https://hamgit.ir/hamdars/mahfel/-/issues/11) | Session roster performs N+1 lookups and a presign per participant | Phase 3 |
| [#12](https://hamgit.ir/hamdars/mahfel/-/issues/12) | Pagination offset can overflow on large page values | Phase 1 |
| [#13](https://hamgit.ir/hamdars/mahfel/-/issues/13) | Capability introspection and moderation enforcement disagree for global roles | Phase 1 |
| [#14](https://hamgit.ir/hamdars/mahfel/-/issues/14) | Session roster is readable by blocked or lobby-waiting non-members | Phase 1 |
| [#15](https://hamgit.ir/hamdars/mahfel/-/issues/15) | session_participation aggregation is O(n^2) | Phase 3 |
| [#16](https://hamgit.ir/hamdars/mahfel/-/issues/16) | Chat slow-consumer eviction skips disconnect cleanup and leaks open participation | Phase 4 |
| [#17](https://hamgit.ir/hamdars/mahfel/-/issues/17) | Collaboration WebSocket bypasses the connection cap and has no idle timeout | Phase 1 |
| [#18](https://hamgit.ir/hamdars/mahfel/-/issues/18) | Main WebSocket has no inbound rate limit; Broadcast amplifies every frame | Phase 1 |
| [#19](https://hamgit.ir/hamdars/mahfel/-/issues/19) | Chat fanout issues one event_head query per subscriber per tick | Phase 4 |
| [#20](https://hamgit.ir/hamdars/mahfel/-/issues/20) | Moderation and presence frames are replica-local; cross-replica targets never see them | Phase 3 |
| [#21](https://hamgit.ir/hamdars/mahfel/-/issues/21) | Speaker projection and collaboration fanout maps are never pruned | Phase 4 |
| [#22](https://hamgit.ir/hamdars/mahfel/-/issues/22) | Collaboration checkpoints are never pruned and store full snapshots | Phase 4 |
| [#23](https://hamgit.ir/hamdars/mahfel/-/issues/23) | A transient Redis error during presence renewal disconnects every socket | Phase 2 |
| [#24](https://hamgit.ir/hamdars/mahfel/-/issues/24) | Media placement is re-resolved with two Redis calls on every RTC frame | Phase 2 |
| [#25](https://hamgit.ir/hamdars/mahfel/-/issues/25) | Non-idempotent media create RPCs are retried, orphaning transports/producers/consumers | Phase 2 |
| [#26](https://hamgit.ir/hamdars/mahfel/-/issues/26) | Media control-plane calls have no deadline and can hang the API | Phase 2 |
| [#27](https://hamgit.ir/hamdars/mahfel/-/issues/27) | close_participant leaves stale consumers in the consumer map | Phase 2 |
| [#28](https://hamgit.ir/hamdars/mahfel/-/issues/28) | One job persistence error tears down the AMQP consumer and aborts concurrent jobs | Phase 5 |
| [#29](https://hamgit.ir/hamdars/mahfel/-/issues/29) | Claim recovery only runs while the AMQP relay is connected | Phase 5 |
| [#30](https://hamgit.ir/hamdars/mahfel/-/issues/30) | GetNodeStats always reports participants = 0 | Phase 2 |
| [#31](https://hamgit.ir/hamdars/mahfel/-/issues/31) | Media control gRPC service is unauthenticated | Phase 1 |
| [#32](https://hamgit.ir/hamdars/mahfel/-/issues/32) | Outbox relay ignores the mark_published result | Phase 5 |
| [#33](https://hamgit.ir/hamdars/mahfel/-/issues/33) | BFF request-size limit can be bypassed with chunked bodies | Phase 1 |
| [#34](https://hamgit.ir/hamdars/mahfel/-/issues/34) | BFF does not strip forwarding and hop-by-hop headers | Phase 1 |
| [#35](https://hamgit.ir/hamdars/mahfel/-/issues/35) | BFF allowlist relies on URL normalization for containment | Phase 1 |
| [#36](https://hamgit.ir/hamdars/mahfel/-/issues/36) | Federated logout open redirect via protocol-relative callbackUrl | Phase 1 |
| [#37](https://hamgit.ir/hamdars/mahfel/-/issues/37) | Send-side RTC resources leak when unmount races async setup | Phase 2 |
| [#38](https://hamgit.ir/hamdars/mahfel/-/issues/38) | Chat refetches full history on every incoming event | Phase 4 |
| [#39](https://hamgit.ir/hamdars/mahfel/-/issues/39) | Collaboration/signaling client robustness: parse guard, handshake timeout, duplicate sockets | Phase 2 |
| [#40](https://hamgit.ir/hamdars/mahfel/-/issues/40) | Presentation buffers full file in React state and ICE restart reuses stale parameters | Phase 2 |

## Validation and closure policy

Use focused tests while implementing each change. At each affected-component completion gate, run:

```bash
just backend fmt
just backend lint
just backend test
just frontend fmt
just frontend lint
just frontend typecheck
just frontend test
```

Run only the affected component gates for isolated changes. Backend `test` includes unit,
Testcontainers integration, and doctest checks; report unavailable infrastructure precisely.
Use `just frontend build` for BFF/runtime or Next.js changes and `just frontend storybook-build`
when reusable UI primitives change. For shared contracts run `just proto fmt-check`, `just proto lint`,
and regenerate REST/WebSocket types as appropriate; never hand-edit generated files.
Use `just docs` for documentation and `docker compose config` for deployment/Compose changes.

Add real cross-service scenarios under `e2e/tests` and run `just e2e` against a known application
revision. Existing Chromium/Firefox fixtures auto-approve fake capture devices; denial tests need
an explicit fixture that does not auto-approve. Synthetic camera/microphone input is acceptable,
but signaling, admission, persistence, and SFU traffic must exercise the real services. Two API
replicas and controlled socket routing are required to prove the replica-specific reports.
Record peer counts, decoded audio/video progress, bounded reconnect, durable participation state,
and media/resource counts; screenshots alone do not establish media correctness.

Preserve the four runtime components, PostgreSQL authority, ephemeral Redis coordination, direct
WebRTC, narrow REST BFF, and private media control. ADRs 0017, 0019, 0021, and 0025 govern the
notification, browser-authentication, invitation, and WebSocket-ticket boundaries.

Close an original bug issue only with current revision, reproduction disposition, changed surface,
and relevant passing checks. Update a stale description/checklist transparently and preserve its
history. For milestone issues #71/#175/#176/#179, attach browser and multi-client evidence as well
as local checks. Link original bug resolutions rather than duplicating them in new tasks.

Close M3 and M4 only once their remaining tasks and relevant sign-off regressions pass. M5's seven
feature tasks are already closed; its milestone can close after the chat acceptance checks. Do not
silently infer live production proof from source, unit tests, CI, or a successful deployment.
Worker and unrelated collaboration reports remain separately tracked if their fixes are not needed
for milestone acceptance; all are still included in this plan.

During this planning pass, GitLab was read but no issues or milestones were edited. No application
code was changed and no functional/runtime test was executed. Future execution should update the
original GitLab issues incrementally with evidence and use separate, coherent patches.


## Execution checkpoint — 2026-10-07

Implementation is underway. Milestones remain open until the browser and multi-replica acceptance
criteria above have passed against the changed application.

| Issues | Implemented behavior | Verification at this checkpoint |
| --- | --- | --- |
| #7, #8, #9 | Explicit persisted role updates with a transactional last-manager guard; room-specific lobby lookup/count; atomic admission decision and membership/participant/interval grant. | API role regression; disposable PostgreSQL concurrent demotions, 24 approval/rejection races, and injected interval failure/rollback/retry. |
| #11, #14 | Roster identities require admission or moderation authority and deny active room/session blocks; user/avatar queries are batched and signing concurrency is capped at eight. | Application authorization regression and disposable PostgreSQL batch-query/ready-avatar selection tests. |
| #12, #15 | Pagination rejects unrepresentable PostgreSQL offsets; participation aggregation uses an index while preserving first-seen order, corrections, and session-end clamping. | Boundary and aggregation regressions. |
| #13 | Moderation retains verified principal roles and resolves the same policy as capability introspection. | Global admin/moderator action matrix plus existing member/outsider/self-target denial tests. |
| #25, #26 | Resource creation runs once with a five-second deadline; direct room/source-control calls also have deadlines. | Ambiguous failure is attempted once; an unresponsive operation returns deadline exceeded. |
| #27, #30 | Participant removal prunes owned and producer-dependent consumers; node counts include receive-only actors and deduplicate transports by room/actor. | Real mediasoup lifecycle regression with producers, consumers, and multiple transports. |
| #33–#36 | The BFF bounds actual body bytes, filters proxy headers, validates final upstream paths, and contains logout redirects within the configured origin. | Fourteen focused security regressions. |
| #37, #39 | Late capture/device/producer results are cleaned up after unmount; pending connects coalesce; stale tickets are ignored; collaboration JSON and authentication deadlines are guarded. | Send-media lifecycle and signaling/collaboration regressions. |

The latest complete frontend run passed 247 tests across 76 files. The latest API/application run
passed 280 tests. Workspace Clippy passed with warnings denied. These are local checks, not deployment
or milestone browser acceptance evidence.

Local acceptance infrastructure uses temporary port overrides in `/tmp/mahfel-execution/compose.env`
because the running LMS stack occupies the default PostgreSQL, Redis, RabbitMQ, and HTTPS ports.
Existing volumes were retained. The restored API starts successfully; its running image predates these
changes and is not validation of the patch.

### Additional execution evidence

Session lists now paginate and filter by status throughout REST and workspace callers. The API
pagination regression and real PostgreSQL pagination/checkpoint-retention tests pass. Realtime
limits, bounded collaboration sockets, cross-replica moderation reconciliation, sequenced chat
updates, and private media authentication are implemented. The real worker integration passes
with PostgreSQL, RabbitMQ, and S3, including lease recovery while the broker is unreachable.

The admission/removal/block/session-end media authorization regression, two-registry durable
moderation reconciliation regression, and real mediasoup transport/producer/consumer cleanup
regression pass. Final broad gates and authenticated Chrome/Firefox acceptance are still pending;
these results do not justify closing the milestones yet.
