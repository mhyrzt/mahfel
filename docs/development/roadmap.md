# Engineering roadmap

This roadmap describes the implementation sequence after bootstrap. Roadmap IDs are stable
planning identifiers; [GitHub issues](https://github.com/mhyrzt/mahfel/issues) hold the executable
work records and [GitHub milestones](https://github.com/mhyrzt/mahfel/milestones) group delivery.
Dependencies name roadmap IDs or milestone completion. Capacity and product assumptions must be
validated rather than treated as promises.

See [issue tracking](issue-tracking.md) for the legacy task-to-issue mapping. This document describes
scope and sequencing; GitHub owns current task status and validation evidence.

## M0 — Repository and engineering foundation

**Purpose:** establish a repeatable, observable, production-oriented starting point without product
functionality.

| ID and title | Problem description | Implementation scope | Dependencies | Done criteria |
|---|---|---|---|---|
| M0-01 Repository boundaries | A new project needs clear deployable/module ownership. | Create root layout, exactly four app components, meaningful Rust crates, feature-oriented frontend, ignore/editor files. | None | Tree is documented; no empty speculative crates; app components are web/API/media/worker. |
| M0-02 Rust workspace | Services need compileable shared foundations. | Edition 2024 workspace, pinned stable toolchain, typed config, errors, SQLx migration directory, protobuf generation, mediasoup boundary. | M0-01 | Workspace resolves with lockfile and service shells compile in supported builder. |
| M0-03 Frontend scaffold | UI work needs one consistent toolchain. | Bun, Next App Router, strict TS, Tailwind, shadcn config/button, minimal page, public API URL, health route. | M0-01 | Install, lint, typecheck, and production build pass. |
| M0-04 Service operations | Operators need safe probes before product work. | Separate operations listeners, JSON logs, process/readiness metrics, graceful shutdown, dependency-aware readiness. | M0-02 | `/metrics`, live, and ready behave as documented on 9101–9103. |
| M0-05 Internal media contract | API/media ownership must be enforceable early. | Versioned protobuf and generated tonic bindings; node info/stats implemented; lifecycle methods explicitly unimplemented. | M0-02 | Contract compiles without host protoc and browser has no path to gRPC. |
| M0-06 Production images | Every component needs reproducible runtime packaging. | Four multi-stage Dockerfiles, non-root runtime, cached builds, signals, health checks, native media builder. | M0-02, M0-03 | All images build; build-only tools absent from runtime images. |
| M0-07 Local platform | Cross-service development otherwise diverges. | Compose PostgreSQL, Redis, RabbitMQ, RustFS, coturn, Prometheus, Grafana, Alertmanager; networks, volumes, checks, local env. | M0-04, M0-06 | One command starts healthy stack; only necessary public/loopback ports are mapped. |
| M0-08 Documentation and gates | Decisions and validation need a shared record. | MkDocs, architecture, ADRs, commands, testing conventions, roadmap, exact validation log. | M0-01–M0-07 | Strict docs build; all requested gates recorded; clean local commit exists. |

**Milestone done:** one command starts the stack; all four images and both workspaces build; frontend
lint/typecheck, Rust fmt/Clippy/tests, and strict docs pass; Prometheus scrapes all Rust services and
Grafana uses the provisioned datasource.

## M1 — Control-plane foundation

| ID and title | Problem description | Implementation scope | Dependencies | Done criteria |
|---|---|---|---|---|
| M1-01 Migration runner and pools | Durable features need controlled schema lifecycle and bounded connections. | SQLx migration command/startup policy, pool configuration, timeouts, health/metrics, disposable DB test harness. | M0 | Forward migration runs once safely; pool failure is observable; integration test provisions schema. |
| M1-02 User/room/session core | Routes need durable identifiers and invariants without a complex schema. | Minimal users, rooms, classroom sessions, timestamps/statuses, repositories and migrations. | M1-01 | Repository tests create/read each model and data survives process restart. |
| M1-03 HTTP contract foundation | Clients need stable errors, pagination, validation, and versions. | `/v1`, request IDs, JSON error envelope, validation/extractors, pagination convention, CORS/security middleware. | M0, M1-02 | Contract tests cover success/error/request ID; invalid input never becomes generic 500. |
| M1-04 Room endpoints | A vertical durable slice is required before richer features. | Authorized create/get room handlers, service layer, repository calls, API tests. | M1-02, M1-03 | Room is created/read via API and remains after restart. |
| M1-05 WebSocket gateway | Signaling and realtime capabilities need a scalable connection boundary. | Upgrade route, connection lifecycle, heartbeat, typed envelope/versioning, bounded queues, graceful close, metrics. | M1-03 | Multiple clients connect, exchange a test event, detect disconnect, and expose aggregate metrics. |
| M1-06 Redis coordination adapter | Replicas need ephemeral shared coordination without making Redis authoritative. | Async pool, namespaced keys/TTL, pub-sub strategy, failure classification, health metrics, tests. | M1-01 | Ephemeral key expires; outage behavior is documented/tested; no durable model exists only in Redis. |
| M1-07 Authorization/OpenAPI decisions | Later routes need consistent policy and discoverability. | Authorization service interface, deny-by-default convention, development principal, ADR/decision for OpenAPI generation. | M1-03 | Room handler uses authorization seam; development identity is explicit; OpenAPI approach recorded. |

**Milestone done:** a room can be created/read, a development/authenticated session can connect to
WebSocket, and room state persists through PostgreSQL restart.

## M2 — Authentication and identity

| ID and title | Problem description | Implementation scope | Dependencies | Done criteria |
|---|---|---|---|---|
| M2-01 Identity schema | One Mahfel user may have multiple login identities. | Users, identities `(issuer, subject)`, verified emails, avatar source, local credential table, uniqueness/migrations. | M1 | Constraint tests prevent identity takeover and allow multiple identities per user. |
| M2-02 Local password lifecycle | Local users need secure credential creation and verification. | Argon2id parameters, OS randomness/salts, constant-time verification through maintained crate, password change/reset token boundaries. | M2-01 | Known-vector/rehash tests pass; plaintext/password material is never logged or stored. |
| M2-03 Session security | Browser authentication needs revocation, expiry, and CSRF-safe transport. | Decide opaque session versus signed token, secure cookie flags, rotation/revocation, CSRF approach, session store and middleware. | M2-02 | Login/logout/expiry/rotation tests pass and stolen old session is rejected after rotation. |
| M2-04 OIDC provider abstraction | Keycloak and future issuers must not leak vendor logic into handlers. | Discovery, PKCE/state/nonce, callback validation, issuer config, claim normalization, clock/JWK refresh behavior. | M2-01, M2-03 | Fake issuer integration covers success, bad nonce/state/signature, and key rotation. |
| M2-05 Keycloak integration | Target deployments need a verified compatible provider. | Realm/client settings, redirect/logout flows, avatar claim mapping, local Compose or test-realm guide. | M2-04 | Real Keycloak login/logout works and expected claims map deterministically. |
| M2-06 Identity linking | Existing users need safe account association across providers. | Explicit linking ceremony, verified-email policy, collision handling, unlink safeguards and audit events. | M2-03, M2-04 | Same user owns multiple identities; implicit unverified-email linking is impossible; tests cover conflict. |
| M2-07 RBAC and classroom roles | Authentication alone does not enforce classroom actions. | Global admin plus teacher/student/moderator role model, policy checks, audit context, role assignment constraints. | M1-07, M2-03 | Route/service tests prove deny-by-default and role-specific allow/deny matrix. |
| M2-08 Avatar resolution | OIDC avatar URLs and custom uploads need one safe response strategy. | Claim URL validation/proxy decision, custom avatar object metadata, precedence/fallback and cache policy. | M2-04, M6 design input | Strategy documented; unsafe schemes rejected; endpoint contract tested. |
| M2-09 Service principals | External systems such as lms-core need to own and manage rooms without impersonating a user. | Machine actor distinct from users, service-authentication mechanism, provisioning/rotation/revocation, room ownership and management by a service, ADR. | M2-01, M2-03, M2-07 | Service principal acts as owner/manager without a user row; revocation enforced; matrix tests deny by default. |

**Milestone done:** local and Keycloak login work, multiple identities map to one Mahfel user, and
classroom role enforcement is integration-tested.

## M3 — Media-plane proof of concept

| ID and title | Problem description | Implementation scope | Dependencies | Done criteria |
|---|---|---|---|---|
| M3-01 Worker supervision | Native worker crashes/resource limits need explicit lifecycle behavior. | Configurable worker count/settings, spawn supervision, failure metrics, readiness, shutdown and restart policy. | M0 | At least one worker is required for readiness; induced failure is detected and documented. |
| M3-02 Router/room lifecycle | Every live room needs isolated mediasoup resources. | Codec capabilities, router create/lookup/delete, ownership map, idempotent gRPC commands and cleanup. | M3-01, M0-05 | Create/delete is idempotent; leaks are absent in repeated lifecycle test. |
| M3-03 WebRtcServer/transports | Browsers need reachable ICE/DTLS transports on predictable ports. | One WebRtcServer per worker, UDP/TCP listen infos, announced address, send/receive transport lifecycle, ICE/DTLS parameters. | M3-02 | Browser transport connects over direct UDP; TCP fallback behavior is recorded. |
| M3-04 Producer lifecycle | Teachers must publish microphone/camera safely. | Produce validation, audio/video codecs, simulcast/SVC parameters, pause/resume/close, ownership and cleanup. | M3-03 | Real browser publishes mic/camera; invalid participant/transport requests fail cleanly. |
| M3-05 Consumer lifecycle | Students need server-selected consumable tracks. | Capability validation, consumer creation paused-first, resume/pause/close, preferred layers and producer-close propagation. | M3-04 | Multiple browsers consume teacher tracks and cleanup follows disconnect. |
| M3-06 Signaling integration | Browser must never call private media gRPC directly. | API WebSocket signaling commands, authorization, correlation/timeouts, media client retries/status mapping. | M1-05, M2-07, M3-03–M3-05 | End-to-end signaling works; gRPC remains private; media bytes bypass API. |
| M3-07 Frontend RTC client | Device and transport state must survive normal browser behavior. | `mediasoup-client`, device enumeration/permissions, send/receive transports, track stores, error UX. | M3-06 | Two supported browsers publish/consume and device denial has actionable UI. |
| M3-08 Screenshare/reconnect | Core classroom media needs presentation and transient recovery. | Display track lifecycle, replace/stop signals, connection-state handling, bounded reconnect and full rejoin fallback. | M3-07 | Screen share starts/stops; brief disconnect recovers or produces explicit rejoin outcome. |
| M3-09 TURN and media metrics | Restricted paths and capacity need evidence. | Short-lived TURN credentials, forced relay test, aggregate mediasoup stats exporter and bounded labels. | M3-05–M3-08, M0-07 | TURN fallback works; required room/participant/transport/bitrate/loss/RTT metrics are scrapeable. |

**Milestone done:** several real browsers join; teacher audio/video and screen share reach students;
direct UDP and forced TURN work; metrics exist; no media traverses API.

## M4 — Realtime session engine

The engine is generic realtime collaboration (ADR 0020). Vocabulary is Room, Session, Participant,
participation record, capability, and admission policy. Education concepts are an integration mapping,
not core types.

| ID and title | Problem description | Implementation scope | Dependencies | Done criteria |
|---|---|---|---|---|
| M4-01 Session vocabulary and lifecycle | Persistent rooms and live sessions have different lifetimes, and the live aggregate is named for one vertical. | Rename ClassroomSession to Session (table sessions, amended migration 0002), session state machine, one-live-session-per-room rule, ownership by room manager or service principal, start/end/cancel APIs and audit. | M1, M2-07, M3-06 | Invalid transitions rejected; start/end persists and is audited; one live session per room enforced. |
| M4-02 Participant lifecycle and reconnect identity | Connections are transient but session presence is durable. | Join authorization, durable per-session participant identity, explicit state machine, reconnect generation, multi-device policy, leave/removal cleanup. | M4-01, M2-07, M3-06 | Reconnect does not duplicate a participant; state transitions and multi-device policy are tested. |
| M4-03 Participation records | Presence duration must be durable and correctable, not derived from ephemeral state. | Join/leave intervals per participant per session, aggregation to connected duration, reconnect grace policy, audited corrections, session and participant queries. | M4-02 | Intervals never overlap and aggregate correctly; no open interval survives session end. |
| M4-04 Session capabilities and authorization | Moderation and session actions must not depend on a fixed vertical role enum. | Capability set derived from global roles, room membership, and service principals; one policy seam for REST and WebSocket; deny-by-default matrix; documented education mapping. | M2-07, M2-09, M4-01 | Allow/deny matrix covers member, manager, moderator, admin and service principal; no handler hardcodes roles. |
| M4-05 Moderation controls | A session needs enforceable, role-aware moderation without vertical-specific roles. | Mute request vs enforced mute, remove/block, publish/screen gating, WebSocket propagation, audit events and media-control enforcement. | M4-04, M4-02, M3-06 | Unprivileged participant cannot bypass policy; controls propagate and are integration-tested. |
| M4-06 Admission and lobby | A room may need controlled entry instead of open join. | Room admission policy open or lobby, bounded waiting state, admit/reject capability checks, enforcement in REST and WebSocket join, audit. | M4-02, M4-04 | Lobby policy blocks media until admitted; decisions are enforced server-side and audited. |
| M4-07 Active speaker | UI layout needs a stable aggregate speaker signal. | mediasoup observer, smoothing/rate limits, authorized WebSocket event, cleanup and bounded metric without participant labels. | M3-05, M4-02 | Multiple speakers update UI without event flood; disconnect cleanup works. |
| M4-08 Media node registry | Scheduler needs fresh capacity and lifecycle state. | Redis heartbeat schema/TTL, node identity/address/capacity/draining, private authenticated update path and metrics. | M1-06, M3-09 | Dead node expires; draining state persists for heartbeat lifetime; spoofed update rejected. |
| M4-09 Session placement scheduler | Every participant in a session must reach the same capable node. | Atomic placement lease, weighted capacity policy, sticky lookup, race/failure behavior, durable diagnostic record. | M4-08 | Concurrent joins select one node; overload/draining excludes node; Redis race tests pass. |
| M4-10 Frontend session state and roster | The browser must hold stable participant state across reconnects. | WebSocket session connect/join/leave, roster from stable identities, reconnect reconciliation, lobby state, capability-gated controls. | M4-02, M4-06, M3-07 | Roster survives reconnect without duplicates; lobby and capability gating verified. |
| M4-11 Frontend moderation and admission UI | Operators need capability-gated controls that match server policy. | Mute/enforce/remove/block and publish/screen controls, admission queue with admit/reject, server-authoritative feedback. | M4-05, M4-06, M4-10 | Controls appear only with capability; server rejects bypass; admission queue updates live. |
| M4-12 Frontend session lifecycle and active speaker UI | Lifecycle and speaker state need a coherent interface. | Ad-hoc/scheduled session creation with optional title and duration, start/end/cancel controls, scheduled/live/ended treatment, active-speaker layout, post-session participation summary. | M4-01, M4-03, M4-07, M4-10, M4-16 | State drives UI; schedule is advisory; active speaker layout is stable; ended session shows participation summary. |
| M4-13 Invite link model and management API | Rooms and sessions need a shareable, retractable entry path beyond membership. | Globally unique `xxx-yyyy-zzz` codes, one target (room or session), 7-day default expiry bounded by policy, active/revoked/expired status, create/list/revoke/rotate/regenerate gated by capability, plaintext storage with rate-limited redemption and audit. | M4-01, M4-04 | Codes are unique and normalizable; expired/revoked rejected; management deny-by-default; codes never logged; rotation invalidates atomically. |
| M4-14 Link join and approval flow | A shared link must not grant immediate access. | Redeem by code into a waiting state unless already an explicit member, admit/reject by capability, session-link approval grants session participation only, room-link approval grants durable membership, rate-limited and non-enumerating failures, audit. | M4-13, M4-06, M4-04, M4-02 | Explicit members bypass confirmation; session links add no membership; room links do; no media before approval; failures audited. |
| M4-15 Frontend invite links UI | Holders and managers need a usable link experience. | Create/list/copy/revoke/regenerate links for rooms and sessions, capability-gated visibility, join-by-code with waiting/confirmation state and explicit expired/revoked/rejected states. | M4-13, M4-14, M4-10, M4-04 | Management hidden without capability; waiting state shown until resolution; server remains authoritative. |
| M4-16 Session scheduling and title | A session may need a planned time, a duration, and its own title. | Optional single-occurrence schedule (start instant and advisory duration) and optional session title with room-title fallback, columns on the sessions table, capability-gated and audited; room title stays required. | M4-01, M4-04 | Ad-hoc and scheduled sessions both supported; schedule never gates start; duration is advisory; title falls back to room title. |

**Milestone done:** sessions have a generic lifecycle, an optional schedule and title, and durable
participation records; capabilities and moderation are enforced server-side; admission, active
speaker, node registry, and placement work; invite links for rooms and sessions resolve to a
confirmation flow unless the principal is already a member; the browser roster and controls stay
consistent across reconnects. Engine vocabulary stays generic and an education deployment maps
teacher/student/attendance onto capabilities and participation records.

## M5 — Persistent chat

| ID and title | Problem description | Implementation scope | Dependencies | Done criteria |
|---|---|---|---|---|
| M5-01 Channel/message model | Classroom chat must outlive live sessions. | Classroom/meeting channels, message IDs/content/author/timestamps, tombstones, indexes and repository. | M1, M2 | Messages persist and are scoped to authorized channel. |
| M5-02 History and mutations | Large histories need stable paging and controlled edits/deletes. | Cursor pagination, edit window/policy, soft delete, revision/audit data and API contracts. | M5-01 | Pagination has no gaps/duplicates under inserts; mutation permissions tested. |
| M5-03 Realtime fanout | Multiple API replicas must deliver the same committed events. | Transaction-to-event boundary, Redis pub-sub/stream decision, local connection hub, sequence/dedupe and backpressure. | M1-05, M1-06, M5-01 | Clients on two API replicas receive committed message once; slow client is bounded. |
| M5-04 Reactions/mentions/threads | Classroom discussion needs structured interaction, not opaque text hacks. | Reaction uniqueness, mention validation/notification event, parent/reply constraints and thread queries. | M5-02, M5-03 | Concurrent reactions converge; invalid mentions/parents rejected; thread history paginates. |
| M5-05 Typing/presence/read markers | Ephemeral UX and durable reading state need separate storage semantics. | TTL typing/presence in Redis, aggregate events, durable per-user channel cursor, privacy/scale limits. | M1-06, M5-03 | Ephemeral indicators expire; read marker survives restart; no user IDs appear in metrics. |
| M5-06 Attachments and abuse controls | Chat files and bursts need safe limits. | Attachment metadata link, pending/ready states, rate/size/content limits, moderation hooks. | M6-03, M5-02 | Only authorized ready objects attach; flood and invalid object tests fail predictably. |

**Milestone done:** chat persists after meetings, paginates, fans out across replicas, and enforces
permissions for messages, mutations, structured interactions, presence, and attachments.

## M6 — Files and S3

| ID and title | Problem description | Implementation scope | Dependencies | Done criteria |
|---|---|---|---|---|
| M6-01 Object metadata and ACL | Artifact bytes need authoritative ownership/permission metadata. | Object key strategy, owner/classroom/session scope, status, size/MIME/checksum, ACL service and migrations. | M1, M2-07 | Metadata constraints/ACL matrix tests pass; keys are collision-resistant and non-guessable. |
| M6-02 S3 adapter | Direct transfers need a tested S3-compatible storage boundary. | AWS SDK client config, endpoint/path-style support, timeouts/retries, bucket checks and error mapping. | M0-07 | Signed transfer integration suite passes against disposable RustFS, including bucket CORS and lifecycle configuration. |
| M6-03 Presigned upload | APIs should authorize but not proxy file bodies. | Initiate contract, limits, short expiry, required headers/checksum, completion verification and abandoned cleanup. | M6-01, M6-02 | Browser uploads directly; oversized/wrong checksum cannot become ready metadata. |
| M6-04 Authorized download | Private objects need short-lived controlled access. | Permission check, content disposition, presigned GET, cache policy, audit event. | M6-01, M6-02 | Authorized user downloads directly and unauthorized user receives no usable URL. |
| M6-05 File contexts | Classroom, meeting, chat, avatar, and future recording files need explicit association. | Context/link tables or typed owner model, lifecycle/deletion rules and list APIs. | M6-03, M6-04 | Each context lists only its files; deleting a link cannot orphan bytes indefinitely. |
| M6-06 Validation/security strategy | Client MIME and extensions are untrusted. | Sniffing/quarantine decision, allowed types, malware hook, image normalization, quotas and observability. | M6-03 | Threat model documented; executable/polyglot test fixtures follow chosen policy. |

**Milestone done:** browsers transfer directly with S3, PostgreSQL retains metadata, checksums and
limits are enforced, and every file access is authorized.

## M7 — Collaborative notes and whiteboard

| ID and title | Problem description | Implementation scope | Dependencies | Done criteria |
|---|---|---|---|---|
| M7-01 CRDT document identity | Notes/boards need ownership, type, permission, and version boundaries. | Document metadata/schema, classroom/session association, ACL, lifecycle and protocol version. | M1, M2-07, ADR 0012 | Unauthorized sync rejected; each document type/version resolves deterministically. |
| M7-02 Yrs synchronization gateway | Binary CRDT updates need scalable authenticated transport. | Yrs doc manager, Yjs sync messages over WebSocket, size/rate limits, cross-replica routing. | M1-05, M7-01 | Two clients on different API replicas converge under concurrent updates. |
| M7-03 Persistence/checkpoints | In-memory CRDT state must survive restarts without unbounded replay. | Update log versus snapshot design, atomic checkpoint, compaction, load/recovery and corruption handling. | M7-02 | State restores after restart; compaction preserves state vector; recovery test covers interrupted checkpoint. |
| M7-04 Awareness/presence | Cursors/selections are useful but ephemeral and potentially high-volume. | Yjs awareness relay, TTL cleanup, privacy fields, throttling and no durable persistence. | M7-02, M1-06 | Disconnect/TTL removes awareness and bursts stay within bounded queues. |
| M7-05 Collaborative notes UI | The first document type validates text collaboration end to end. | Yjs frontend provider, note editor choice/adapter, reconnect/resync, permission UX. | M7-02–M7-04 | Multiple users edit simultaneously, reconnect, converge, and survive backend restart. |
| M7-06 Whiteboard object model | Canvas collaboration needs compact mergeable semantic objects. | Object schema/versioning, create/update/delete ordering, transform batching, viewport and basic shapes. | M7-02–M7-04 | Two browsers edit objects concurrently and converge; reload preserves board. |

**Milestone done:** notes and a basic whiteboard collaborate concurrently, reconnect/resync, converge,
and persist through service restart.

## M8 — RabbitMQ worker foundation

| ID and title | Problem description | Implementation scope | Dependencies | Done criteria |
|---|---|---|---|---|
| M8-01 AMQP topology | Jobs need durable versioned routing and ownership. | Exchanges, queues, routing keys, durable declarations, prefetch, schema envelope/version and compatibility rules. | M0, M1 | Topology declaration is idempotent; compatible schema fixture round-trips. |
| M8-02 Reliable publisher | A successful DB request must not silently lose its job. | Confirm-mode channel, mandatory routing, timeout/error metrics, stable job/idempotency IDs. | M8-01 | Unroutable/nacked publish is surfaced; confirmed publish integration test passes. |
| M8-03 Consumer acknowledgements | Workers must distinguish success, transient failure, and poison input. | Manual ack, typed handler result, cancellation, prefetch/concurrency and graceful drain. | M8-01 | Successful job acked; shutdown stops intake and finishes/returns in-flight jobs safely. |
| M8-04 Retry and DLQ | Infinite redelivery hides poison jobs and creates load. | Bounded delayed retries, attempt metadata, exponential policy, DLX/DLQ, operator inspection/replay procedure. | M8-03 | Transient fixture retries then succeeds; poison fixture reaches DLQ after exact limit. |
| M8-05 Idempotency | AMQP is at-least-once, so duplicate execution is normal. | Handler idempotency key/store, atomic claim/result, stale lease and per-job guidance. | M8-03, M1-01 | Duplicate delivery cannot duplicate durable effect; crashed claim recovers safely. |
| M8-06 Transactional outbox | DB commit and broker publish cannot be one transaction. | Outbox table/write with domain transaction, relay leasing, confirms, retry, cleanup and lag metrics. | M8-02, M8-05 | DB change creates durable job atomically; broker outage later drains without loss/duplication effect. |

**Milestone done:** a database mutation creates a durable confirmed job; workers ack manually, retry
safely, dead-letter poison input, and remain correct under duplicate delivery.

## M9 — Recording

| ID and title | Problem description | Implementation scope | Dependencies | Done criteria |
|---|---|---|---|---|
| M9-01 Recording architecture spike | SFU recording format/topology determines reliability and cost. | Measure mediasoup plain-transport taps and GStreamer original-track fragments, timing, resource use, bounded buffering and restart loss; record alternatives in an ADR. | M3, M6, M8 | ADR selects strategy with measured prototype, resource profile, restart limitations and alternatives. |
| M9-02 Recording notice and authorization | Recording requires explicit state and authoritative permissions. | Recording-specific manager/admin/moderator capability with session access, participant notices, default-on scheduled preference and immutable control audits. No consent gate or teacher-specific role. | M2-07, M4 | Unauthorized commands fail; every participant sees current state and failures; Start/Pause/Resume/Stop is audited. |
| M9-03 Recording lifecycle | Control, media, and worker need one durable state machine. | Starting/recording/paused/stopping/finalizing/ready/failed states, idempotent commands and reconciliation. | M9-01, M9-02, M8 | Duplicate start/stop is safe; state recovers/reconciles after API restart. |
| M9-04 Capture and upload | Media fragments must reach durable storage without relying on container disk. | Media tap, bounded spool, segment naming/checksums, multipart/direct S3 upload, backpressure and failure metrics. | M9-01, M9-03, M6-02 | Captured segments appear in S3; storage slowdown is bounded and produces explicit failure. |
| M9-05 Finalization worker | Raw streams need playable synchronized artifacts. | Versioned finalize job verifies original fragments and publishes a synchronized replay manifest with collaboration/chat snapshots, events and archived assets; retries, fencing and temporary cleanup. No video export. | M9-04, M8 | Finalization retry is safe; manifest timing, original fragment readability and checksums are verified; partial coverage is explicit. |
| M9-06 Playback and retention | Finished recordings need authorized access and lifecycle management. | Recording API/UI, presigned playback, duration/status, retention/delete policy and audit. | M9-05, M6-04 | Authorized playback works; unauthorized URL cannot be obtained; retention deletion is tested. |

**Milestone done:** authorized controls and recording notices work; documented restart limitations
hold; the original-track replay bundle lands in private S3 and playback follows Mahfel permissions.

M9-01 through M9-05 capture original media and workspace/session-chat history. M9-06 delivers
authorized interactive playback from the original-track replay bundle and durable, audited retention
deletion. Recreating video, FFmpeg processing, transcoding, composition and MP4/video export are
outside the capture pipeline; a separately approved durable MP4 export capability is documented in
[ADR 0028](../adr/0028-recording-mp4-export.md). The accepted recording strategy and boundaries are
in [ADR 0027](../adr/0027-original-track-recording.md).

## M10 — Transcription

| ID and title | Problem description | Implementation scope | Dependencies | Done criteria |
|---|---|---|---|---|
| M10-01 STT provider contract | Providers differ in audio, streaming, timestamps, languages, and errors. | Provider trait, normalized request/result, credentials, capability discovery, timeout/retry and cost/usage metrics. | M8, M9 | Fake and one real provider pass contract tests; secrets/audio are absent from logs. |
| M10-02 Live versus post-session decision | Latency, quality, cost, speaker mapping and recovery trade off. | Prototype both paths, measure, define initial supported path and failure/reprocessing policy in ADR. | M10-01, M9 | Decision includes measured quality/latency/cost and operational fallback. |
| M10-03 Transcript data model | Searchable speech needs ordered timestamps and revisions. | Transcript/document status, segments, start/end, text, confidence/language, speaker reference and provider provenance. | M1, M10-02 | Constraints prevent invalid ranges; version/reprocess retains provenance. |
| M10-04 Speaker mapping | Media track identity and human speakers can diverge across reconnects. | Track/participant intervals, diarization reconciliation, unknown speaker handling and correction API. | M4-02, M10-03 | Reconnect fixture maps segments correctly; ambiguous speaker remains explicitly unknown. |
| M10-05 Transcription jobs | Long-running STT requires durable orchestration. | Extract/normalize audio, submit/poll/webhook strategy, retries/idempotency, progress, cancellation and DLQ. | M8, M9-05, M10-01 | Recorded meeting produces timestamped persisted transcript after retryable failure. |
| M10-06 Transcript API/UI/export/search | Users need permissioned consumption, not raw provider output. | Paginated segments, seek links, basic Postgres search, correction permissions, VTT/SRT/text export. | M10-03–M10-05 | Authorized transcript is searchable/exportable and timestamps seek correctly. |

**Milestone done:** a recording produces timestamped, permissioned transcript segments accessible
through API/frontend with search and export.

## M11 — AI participant

| ID and title | Problem description | Implementation scope | Dependencies | Done criteria |
|---|---|---|---|---|
| M11-01 AI participant domain | AI must obey membership/identity rules rather than bypass them as a command. | AI identity/provider configuration, participant subtype, join/leave, avatar, status and audit model. | M2, M4 | AI joins logically as visible participant and all actions carry its identity. |
| M11-02 LLM provider abstraction | Vendor-specific APIs and failures should not shape domain logic. | OpenAI-compatible/LiteLLM adapter, model capability config, streaming, usage, timeout/retry/cancel and normalized errors. | M8 | Fake plus configured endpoint pass contract; token/cost/error metrics emitted without prompts. |
| M11-03 Mention workflow | Users need a bounded explicit interaction before autonomous behavior. | Chat mention detection, authorization, durable job, response thread/message, progress/cancel and dedupe. | M5-04, M8, M11-01, M11-02 | Mention produces one AI response under duplicate delivery and respects channel permissions. |
| M11-04 Context assembler | Meeting data is fragmented and may exceed model limits. | Permission-filtered transcript/note/chat/file retrieval, citations/provenance, token budgets, truncation/ranking and freshness. | M6, M7, M10, M11-02 | Fixture proves forbidden context excluded and included claims retain source references. |
| M11-05 Prompt safety boundaries | Untrusted classroom content can manipulate tools or disclose other data. | System policy separation, injection defenses, tool allowlist, content/privacy controls, redaction and adversarial tests. | M11-04 | Cross-classroom and forbidden-tool attempts fail; sensitive data is absent from telemetry. |
| M11-06 AI RBAC and administration | Institutions need control over providers, contexts, quotas, and visibility. | Enablement scopes, allowed models/tools/context types, budgets/rate limits, audit and admin UI. | M2-07, M11-02–M11-05 | Disabled classroom cannot invoke AI; quota and context policies are enforced/tested. |

**Milestone done:** AI joins as an identifiable participant, responds to mentions using only permitted
meeting context, and obeys classroom/admin policies.

## M12 — AI voice

| ID and title | Problem description | Implementation scope | Dependencies | Done criteria |
|---|---|---|---|---|
| M12-01 Spoken request capture | Voice AI needs intentional input without continuous surveillance. | Push-to-ask UI, selected audio interval/tap, consent indicator, cancellation and optional wake-phrase spike. | M3, M11 | User explicitly captures/cancels a question; no continuous audio retained by default. |
| M12-02 Low-latency STT path | Recorded-session STT may be too slow for conversation. | Streaming/short-form provider capability, endpointing, partial/final text, timeouts and language handling. | M10-01, M12-01 | Final transcript reaches AI within measured target and partials never trigger duplicate answers. |
| M12-03 TTS abstraction | AI text must become playable audio across providers. | Provider contract, voice/language config, streaming versus full audio, cache policy, usage/errors and cancellation. | M11-02 | Fake and one provider produce normalized audio; cancellation stops generation promptly. |
| M12-04 AI mediasoup producer | Generated audio must appear as the AI participant, not server noise. | Direct/plain transport injection strategy, RTP pacing/codecs, producer ownership/lifecycle and participant metadata. | M3, M11-01, M12-03 | Multiple browsers hear synchronized AI track and identify it as AI participant. |
| M12-05 Turn interruption and mute | Classroom conversation requires stopping speech and user control. | End-to-end cancel propagation, barge-in policy, queueing, teacher/user mute and producer pause/close. | M12-01–M12-04 | User interrupts in-flight response; muted AI emits no audible track; resources clean up. |

**Milestone done:** push-to-ask speech becomes transcript, LLM response, and cancelable TTS audio from
the visible AI participant.

## M13 — Scalability and resilience

| ID and title | Problem description | Implementation scope | Dependencies | Done criteria |
|---|---|---|---|---|
| M13-01 Stateless API scaling | Hidden in-memory ownership breaks multi-replica behavior. | Audit sessions, WebSockets, collaboration, signaling, fanout and idempotency; externalize/reconstruct required state. | M1–M12 | Adding API replica needs no migration/sticky HTTP assumption and cross-replica tests pass. |
| M13-02 Worker concurrency scaling | Replicas must compete without duplicate effects or shutdown loss. | Per-queue concurrency/prefetch, leases, idempotency audit, backpressure, drain and autoscaling signals. | M8–M12 | Multiple replicas process each effect once; scale-down safely resolves in-flight jobs. |
| M13-03 Database capacity | Replica growth can exhaust PostgreSQL or amplify slow queries. | Global pool budget, PgBouncer decision, statement/lock timeouts, query metrics, index review and overload behavior. | M1, workload profiles | Saturation test sheds/degrades predictably; dashboards expose pool waits and slow operations. |
| M13-04 Redis/RabbitMQ failure drills | Secondary system failures should not cause restart storms or corruption. | Partition/restart tests, capability degradation matrix, reconnect/backoff, circuit behavior and operator runbooks. | M5, M8 | Each injected outage matches documented readiness/data-loss behavior and recovers automatically. |
| M13-05 Media drain and rolling deploy | Live rooms cannot be moved like stateless HTTP requests. | Draining registry state, scheduler exclusion, deadline, termination grace, optional room warning and rollout runbook. | M4-05, M4-06 | New rooms avoid drain; existing rooms finish within policy; node exits gracefully. |
| M13-06 Media node loss/reconnect | Unexpected death drops volatile SFU state. | Detection time, client notification/rejoin, new placement rules, session continuity limits and chaos test. | M3-08, M4-06 | Node-kill scenario reaches documented reconnect outcome without stale placement. |
| M13-07 coturn resilience/capacity | Relay is bandwidth-heavy and a critical fallback. | Multiple instances/DNS strategy, credential independence, port/address allocation, bandwidth limits, drain and outage tests. | M3-09 | Forced-relay test survives intended failure model and capacity limits are measurable. |

**Milestone done:** API/worker scale horizontally, dependency failures follow documented degradation,
and media/coturn nodes support measured drain, rolling replacement, and loss behavior.

## M14 — Load testing

| ID and title | Problem description | Implementation scope | Dependencies | Done criteria |
|---|---|---|---|---|
| M14-01 RTC load harness | Browser-only manual testing cannot size SFU consumers/network. | Scriptable producers/listeners, realistic codecs/layers, deterministic room/user patterns and result capture. | M3, M13 | Repeatable teacher + 25/50/100 listener scenarios run from documented command. |
| M14-02 Multi-classroom/admission tests | One large room differs from many routers/workers. | Concurrent classroom matrix, ramp/steady/drain phases, admission rejection and media worker distribution. | M14-01, M4-06 | Results identify safe per-node rooms/participants/consumers before overload. |
| M14-03 Network impairment tests | Latency/loss/retransmission and TURN can dominate user quality. | Controlled delay/loss/jitter, forced TURN, reconnect, TCP fallback and client/server stats correlation. | M14-01, M3-09 | Repeatable profiles quantify RTT/loss/retransmissions/bitrate and recovery thresholds. |
| M14-04 Control/collaboration load | WebSocket, chat, presence and whiteboard can overload API independently. | Connection ramp, signaling, chat bursts, typing, note/board update bursts and cross-replica fanout. | M5, M7, M13-01 | Sustained/burst limits, latency/error rate, queue/backpressure behavior are documented. |
| M14-05 Resource telemetry | Capacity conclusions need aligned system/application/network evidence. | Collect CPU, RAM, consumers, ingress/egress, PPS, packet loss, RTT, retransmissions and queue/pool state. | M15 foundation, M14-01–M14-04 | Each benchmark emits timestamped machine-readable results and dashboard snapshot. |
| M14-06 Scheduler thresholds report | Hypotheses must become measured admission policy. | Test 4-vCPU/8-GB starting around 300–400 mostly-listener students, analyze bottlenecks/margins and set thresholds. | M14-02–M14-05 | Report explicitly rejects capacity guarantee and derives configurable scheduler limits from data. |

**Milestone done:** repeatable benchmarks and documented results determine measured scheduler
admission thresholds across RTC, TURN, reconnect, WebSocket, chat, and collaboration loads.

## M15 — Observability and operations

| ID and title | Problem description | Implementation scope | Dependencies | Done criteria |
|---|---|---|---|---|
| M15-01 Metrics catalog/budgets | Ad hoc metrics create cardinality and cost problems. | Metric ownership, names/units, bounded labels, histogram buckets, scrape budgets and privacy review. | All implemented milestones | Catalog covers services/infrastructure; cardinality tests reject entity labels. |
| M15-02 API/WebSocket dashboards | Operators need latency, saturation, errors and connection/fanout insight. | RED panels, status classes, pool waits, active sockets, message/backpressure, deploy annotations. | M1, M13 | Dashboard isolates elevated API latency cause and affected route/status class. |
| M15-03 Media quality/capacity dashboard | A healthy process can still deliver bad calls. | Workers/routers/transports/producers/consumers, traffic/PPS, bitrate, loss, retransmission, RTT, capacity/drain. | M3, M14 | Operator identifies overloaded node and distinguishes CPU/network/client quality signals. |
| M15-04 Infrastructure/worker dashboards | Queue backlog and dependency saturation need actionable views. | Rabbit depth/rates/unacked/DLQ, worker jobs/failures, Redis, Postgres pool/query, S3 latency/errors and coturn. | M6, M8, M13 | Operator diagnoses queue backlog, worker failure, PostgreSQL saturation, or S3 errors. |
| M15-05 OpenTelemetry correlation | Cross-service latency cannot be reconstructed from unrelated logs. | W3C trace context over HTTP/WebSocket/gRPC/jobs, OTLP exporter config, sampling, Tempo path and request/job IDs. | M1, M8 | One classroom control/job flow is traceable end to end with correlated JSON logs. |
| M15-06 Alerts and runbooks | Dashboards alone do not notify or prescribe action. | SLO-derived alerts, inhibition/routing, media overload, queue age, DLQ, pool saturation, scrape failure and linked runbooks. | M15-02–M15-05 | Synthetic faults fire expected alert without storm and runbook leads to verified diagnosis. |
| M15-07 Error/AI usage telemetry | Exceptions and provider spend need bounded privacy-aware visibility. | Sentry integration boundary, scrub rules, AI token/cost/latency/error aggregates, quotas and provider status. | M11, M12 | Test error correlates without prompt/audio/secret leakage; aggregate AI usage matches fixtures. |

**Milestone done:** an operator can diagnose elevated API latency, overloaded/bad-quality media,
queue/worker failure, PostgreSQL saturation, S3 errors, and AI usage from linked telemetry/runbooks.

## M16 — Production deployment

| ID and title | Problem description | Implementation scope | Dependencies | Done criteria |
|---|---|---|---|---|
| M16-01 Platform deployment/config | Development configuration is not a production deployment specification. | Production Compose definitions and deployment targets, images, private/public ports, external dependencies, runtime secrets, resources, healthchecks and scrape discovery. | M13, M15 | Deployment configuration validates and contains no plaintext secrets; durable application state remains in PostgreSQL and S3. |
| M16-02 HTTP scaling rollout | Web/API/worker need safe horizontally scaled release behavior. | Proxy domains/TLS/CORS, API replicas, worker concurrency, resource limits/autoscaling, migration job and rollback. | M16-01 | Two API and two worker replicas pass functional/rolling deployment tests. |
| M16-03 Media public networking | SFU candidates fail if stable addresses or port preservation are wrong. | Per-replica stable IP decision, 40000+ UDP/TCP service, announced address injection, firewall and flow affinity. | M16-01, platform answers | Two nodes receive reachable candidates; external/container ports match; direct UDP verified. |
| M16-04 UDP platform qualification | LB throughput/PPS/source-IP behavior is currently unknown. | Packet capture and load tests for port preservation, sustained bandwidth, PPS, loss, source IP and generic UDP balancing. | M16-03, M14 | Evidence answers every open platform question and sets supported topology/limits. |
| M16-05 Production coturn | Local static credentials/no TLS are unsafe for production. | Stable public IP/DNS, TLS 5349 and optional 443, time-limited credentials, relay range, bandwidth/resources and monitoring. | M16-01, M13-07 | Forced TURN works with valid TLS/auth and relay load stays within measured limits. |
| M16-06 Media placement/draining rollout | Scheduler identity must match platform replica/network lifecycle. | Registry address publication, node resources, probes, drain hook/grace period, autoscaling constraints and rollback. | M16-03, M4-06, M13-05 | New rooms distribute across two nodes; rolling removal drains without new placement. |
| M16-07 Production acceptance/runbooks | A green deployment is not proof that classroom paths work. | End-to-end classroom, direct UDP, forced TURN, WebSocket, file transfer, job, observability, backup/restore and incident/rollback checks. | M16-02–M16-06 | Signed test report passes all paths and documents owners, rollback, limitations and capacity. |

**Milestone done:** a production-like environment supports direct UDP and TURN, scaled
API/worker and at least two media nodes, with verified platform limits and operational
runbooks.
