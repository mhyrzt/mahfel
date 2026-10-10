# M3–M5 implementation and acceptance

Implementation: 2026-10-07; final frontend and chat acceptance: 2026-10-09.
Baseline: `7938ae8` with uncommitted implementation changes.
The [completion plan](milestone-completion-plan.md) supplies the original scope and ordering.
This report describes local source, native-service and browser evidence; it does not claim a
commit, push, CI run or production rollout.

## Original issue resolutions

| GitLab issue | Resolution | Focused evidence |
| --- | --- | --- |
| #1 | Revalidate durable admission on every RTC command and enforce connection-owned resources. | API socket tests deny unjoined, sequential and foreign-resource requests; application tests cover removal and readmission. |
| #2 | Preserve the existing last-device lifecycle locks and Redis presence implementation. | API regression proves the first disconnect leaves participation open and the last closes it; Chrome closes another device on API replica 2 while original decoded media continues. |
| #3 | Preserve existing single-session join policy and cleanup. | API socket regression rejects repeated and different-session joins without adding an orphan participant. |
| #4 | Replace the unbounded generation map with a process-wide atomic counter. | Backend unit suite and source audit; no per-session generation entries remain. |
| #5 | Connection teardown closes owned transports; removal closes every target transport and dependent media. | Native SFU lifecycle test, browser leave/reload/rejoin and second-device teardown. |
| #6 | Drop transport/producer/consumer map guards before awaiting SFU operations and revalidate parents after creation. | Native lifecycle and room-delete/create race regression. |
| #7 | Persist membership updates through a dedicated store operation; serialize last-manager decisions. | Application role matrix and PostgreSQL persistence/last-manager regression. |
| #8 | Include room identity in room admission lookup and count queries. | PostgreSQL requests for the same actor in two rooms remain isolated. |
| #9 | Commit conditional approval, membership/participant grant and interval atomically. | PostgreSQL competing approve/reject decisions and injected interval failure/rollback/retry. |
| #10 | Paginate and optionally filter session lists, updating REST contracts and all callers. | API and PostgreSQL page boundaries, older live sessions, invalid/extreme inputs; frontend history navigation tests. |
| #11 | Batch user, membership and avatar metadata lookup; sign needed avatar URLs with concurrency eight. | PostgreSQL batch-avatar fixture and API roster tests. Signing remains per ready avatar; database lookup count no longer grows with participant count. |
| #12 | Check pagination offsets against PostgreSQL i64 limits. | Boundary and overflow unit/API regressions. |
| #13 | Pass the verified principal and global roles through moderation enforcement. | Admin/moderator/member/outsider capability and action tests. |
| #14 | Deny roster discovery for waiting, blocked and unauthorized callers. | Application authorization matrix and API roster denial tests. |
| #15 | Aggregate participation intervals through keyed maps while preserving corrections and terminal clamping. | Participation regression and PostgreSQL interval checks. |
| #16 | Signal slow-consumer closure while retaining the participation cleanup handle. | Two-hub stalled-chat regression and existing disconnect tests. |
| #17 | Apply shared connection capacity, idle heartbeat and bounded writes to collaboration sockets. | Backend gateway/connection checks and source audit. |
| #18 | Bound inbound frame bursts/rate and disable debug broadcast amplification. | Limiter tests and API broadcast rejection regression. |
| #19 | Batch subscriber authorization and event reads per room; reuse the authorized set for presence. | PostgreSQL authorization batching and multi-replica chat; no subscriber-by-subscriber event-head lookup remains. |
| #20 | Deliver moderation through Redis with independent PostgreSQL reconciliation. | Chrome host/learner on separate API replicas: approval, mute/unmute and explicit removal; reconciliation tests cover lost-notification recovery. |
| #21 | Prune idle speaker projections and zero-subscriber collaboration fanout entries independently of Redis health. | Backend suite and explicit lifetime/source audit. |
| #22 | Lock document checkpoint writes and retain the newest three full snapshots. | PostgreSQL retention/reconstruction regression; journal replay remains intact. |
| #23 | Retry bounded renewal failures within the presence lease; immediately reject an expired lease. | Renewal-budget tests and Redis coordination integration. |
| #24 | Reuse per-connection media placement for at most two seconds and clear it on teardown. | Backend suite and source audit; durable authorization is still checked for every command. |
| #25 | Do not retry ambiguous non-idempotent media creation. | Creation-attempt regression. |
| #26 | Apply bounded media-control deadlines. | Deadline regression, full backend suite and live authenticated RPC. |
| #27 | Prune owned and dependent consumers when participants/producers close. | Native SFU lifecycle verifies exact resource counts and repeated cleanup. |
| #28 | Isolate individual delivery failures and requeue persistence failures. | Real PostgreSQL/RabbitMQ/storage test injects one completion failure while another delivery succeeds, then proves recovery. |
| #29 | Recover expired claims in an independent task before broker connection. | Worker integration proves recovery while the broker is unreachable. |
| #30 | Count actors with live send or receive transports once per media room. | Native SFU test covers receive-only actors, multiple devices and multiple rooms. |
| #31 | Authenticate private media gRPC with a configured constant-time bearer credential. | Auth unit tests and live RPC: missing/wrong credential return UNAUTHENTICATED; matching credential succeeds. |
| #32 | Honor lost publication leases and count only successful durable confirmations. | PostgreSQL wrong-lease regression and worker integration. |
| #33 | Bound actual streamed BFF body bytes before upstream dispatch. | Missing/dishonest-length, chunked overflow and cancellation tests. |
| #34 | Strip hop-by-hop, Connection-nominated and client forwarding headers. | Boundary header regressions. |
| #35 | Validate allowlisted normalized paths and final upstream origin. | Encoded traversal, separators and containment regressions. |
| #36 | Restrict logout destinations to the configured origin. | Protocol-relative, external and malformed callback regressions. |
| #37 | Fence async capture, device and producer setup by generation; close late resources. | Media-hook unmount/capture regressions and browser teardown. |
| #38 | Apply ordered chat deltas, resume sequence gaps and fetch reaction/attachment metadata only for affected messages. | Unit duplicate/gap/scope tests and cross-replica browser create/edit/reload/delete; ready attachment commits a durable snapshot in PostgreSQL. |
| #39 | Make signaling connection single-flight; fence stale callbacks and bound authentication handshakes; close malformed collaboration frames. | Signaling/collaboration timeout, duplicate and parse regressions. |
| #40 | Give PDF.js the signed URL and reuse its document; replace stale-ICE reuse with at most three fresh signaling rejoins. | Frontend checks and Chrome/Firefox controlled transport-failure recovery with real decoded media. |

Issue numbers in this table refer to the original GitLab reports, whose descriptions and history are
preserved. Resolutions already present at the baseline are verified rather than reimplemented.

On 2026-10-09, the 40 original reports and four remaining milestone tasks were closed with
`status::Done` and individual resolution/validation notes. A fresh GitLab audit verified all
44 closures and preservation of the other 192 issues. The original bug descriptions and migrated
original records remain intact. M3, M4 and M5 were closed after checking zero open issues:
respectively 9/9, 16/16 and 7/7 issues closed. Other milestones remain unchanged.

## Milestone acceptance

- M3 / issue #71: authenticated Chromium and Firefox publish and consume decoded camera/audio
  through the actual SFU with clients on two API replicas. Chromium also verifies decoded screen
  sharing, permission denial/grant retry, overlapping speakers, a controlled transport-failure
  recovery, reload/rejoin, another device and teardown. Recovery uses fresh signaling and media
  resources; local publication requires the participant to enable capture again.
- M4 / issue #175: lobby admission precedes media access. A waiting manager can access the admission
  queue and explicitly approve their own admission under the existing lobby policy. Cross-replica
  enforced mute, unmute and removal reach the learner; unauthorized actions remain server-gated.
- M4 / issue #176: browser creation, title/duration clear, scheduled/live/ended/cancelled views,
  deliberate invalid-transition errors and PostgreSQL participation summaries pass. Continuous
  synthetic RTP drives two simultaneous speaker highlights without reordering tiles. Persian RTL
  scheduling at 390px width supports keyboard activation and has no horizontal overflow.
- M4 / issue #179: copied room URLs prefill the code while preserving explicit confirmation; room
  approval, rotation, revocation and scheduled-session link management pass. Managers see link
  statuses. Ordinary holders receive the same explicit unusable-code state for expired, revoked and
  unknown codes, as required by ADR 0021; authenticated request rejection remains explicit.
- M5: persistent multi-replica room chat passes create, edit, reload and delete without fetching
  full history on each event. Attachments retain their own bounded metadata-refresh path and
  transactionally recorded readiness notifications.

## Validation boundaries

The backend unit gate passed 416 tests. The full Testcontainers/native integration gate passed 447
with two intentional ignores: the ten-minute recording resource benchmark and a subprocess helper.
The separate full recording-export renderer test is opt-in and was not enabled; recording export
was not part of these milestone or bug resolutions. The attachment readiness correction was added
later and revalidated with PostgreSQL, static analysis and the backend unit gate.

Frontend checks include formatting, strict ESLint, TypeScript, 254 unit/component tests and
production build. The final Chromium chat acceptance also verifies an uploaded attachment becomes
visible on both API replicas, then edit, reload and delete, without full-history event refreshes.
The browser suite is `e2e/tests/milestone-acceptance.spec.ts`; it requires development identities,
real API/SFU/storage services and explicit routing of the second client to another API replica.
Set `MAHFEL_E2E_AUDIO_FIXTURE` to a continuous WAV fixture for overlapping-speaker acceptance.
Fixtures use synthetic capture devices; signaling, persistence and media transport are real.

Acceptance ran against disposable source API replicas and a native source SFU, a disposable
Keycloak realm and proxy, and a separate storage bucket. Existing Compose application images were
not treated as evidence for this patch. Source acceptance artifacts are under
`/tmp/mahfel-execution/acceptance/`; these local artifacts are not durable CI attachments.
The earlier October 7 artifacts were lost on session restart; the October 9 chat artifacts are fresh.
TURN-only networking, packet-loss recovery, production networking, CI and deployment are not
established by this local check. No existing deployment was replaced.

Private media authentication requires setting the same strong control credential in API and media
configuration before a coordinated rollout; the committed local fixture is development-only.
