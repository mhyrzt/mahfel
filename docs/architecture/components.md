# Components

## `mahfel-web`

An independently deployed Next.js App Router application built with Bun, TypeScript, Tailwind CSS,
and shadcn/ui. It eventually owns all user and administration interfaces, device control,
`mediasoup-client`, and Yjs integration. It is the browser OIDC client (NextAuth with Keycloak) and
exposes a narrow BFF for authenticated API calls (ADR 0019). It does not contain authoritative
backend logic.

## `mahfel-api`

The Rust control plane and authoritative application service. It owns REST, WebSocket signaling,
identity integration, authorization, users, rooms and sessions, participation records, persistent
chat, file metadata and presigning, collaborative-state orchestration, media-node scheduling, PostgreSQL
persistence, Redis coordination, RabbitMQ publishing, and the gRPC media client.

It exposes the versioned `/v1` control-plane API (status, identity, room, and session endpoints)
plus a WebSocket gateway at `/v1/ws`. The REST contract is generated from the transport code and
served as OpenAPI at `/v1/openapi.json`, with its interactive UI on the private operations listener
(ADR 0015). Higher-level product surfaces remain deliberately deferred.

OIDC is provided by Keycloak for both local development and production (`auth.mahfel.localhost`
locally). The browser authenticates through `mahfel-web` (NextAuth with the Keycloak provider) and a
narrow BFF (ADR 0019). The API is a resource server: it builds the principal from the `app_user_id`
claim without an identity lookup, and `POST /v1/auth/provision` provisions the local user and links
its id onto the Keycloak identity. The API also manages the realm through the Keycloak Admin API
using the client-credentials grant of a dedicated service account rather than an interactive
administrator account. Identity linking, role mapping, and logout remain M2 implementation concerns;
the local realm fixture is described in `deploy/keycloak/README.md`.

Avatar claims are display hints, not files hosted by Mahfel. The API accepts only absolute HTTPS
URLs with a host and no embedded credentials, returns them directly, and never fetches or proxies
provider images. An invalid optional `picture` claim is discarded without failing login. Profile
responses use `Cache-Control: private, no-store`. During provisioning, a user-owned
`users.avatar_url` overrides the provider picture; otherwise the provider picture is used, then the
profile has no avatar. Both profile routes resolve the user-owned override before the provider
picture. Authentication itself still builds the principal from the token; only the profile response
reads the local user record. Custom uploaded avatar objects use the existing `avatar` file context,
keyed by actor ID; their metadata remains in PostgreSQL and bytes remain in private object storage.
File upload and download continue through the normal authorized file endpoints, whose signed URLs
are short-lived and non-cacheable.

The API also owns service principals: first-class machine actors that authenticate with
Mahfel-issued opaque API keys (`mhf_` followed by hex, stored only as a SHA-256 digest) rather than
Keycloak client credentials (ADR 0022). Users and service principals share one PostgreSQL id space
(`mahfel.actors`), and rooms and memberships reference actors, so a service can own a room and hold a
`manager` or `member` role without impersonating a user. Administrators provision, rotate, and
revoke principals under `/v1/admin/service-principals`; the API-key path and the JWT path produce
the same `Principal`, and service principals carry no global roles by default. `mahfel-media` and
`mahfel-worker` do not own service principals or API keys.

The first control-plane persistence boundary is intentionally small: actors (users and service
principals), users, rooms, and sessions have UUID identifiers, explicit lifecycle statuses, and UTC
timestamps in PostgreSQL.
A room is a durable container with `public` or `invite_only` visibility and room-scoped memberships
whose roles are `manager` and `member`. A room's identity is its id alone, so titles need not be
unique. A `Session` (table `mahfel.sessions`) is a live occurrence of a room with an explicit state
machine — `scheduled` to `live` to `ended`, or `scheduled` to `cancelled` — and at most one session
per room may be `live` at a time, enforced by a partial unique index so concurrent starts cannot both
win. Scheduling and starting require `manage_session`; ending and cancelling remain manager-only
(`POST /v1/rooms/{room_id}/sessions` and `/v1/sessions/{session_id}/{start,end,cancel}`). Service
principals participate by managing their rooms, and every lifecycle action writes a structured
audit log. Reading a
session's optional title, planned start, and planned duration is part of its response. Without a
custom title, the response uses the room's current title. Planned duration is 1–10080 minutes and
informational: it never starts or ends a session. `PATCH /v1/sessions/{session_id}` changes or
clears each detail independently, including after end or cancellation, subject to `manage_session`;
the update writes a structured audit event and touches no lifecycle columns. Reading a
room's sessions is manager-scoped, while a session a caller may join is readable when resolving its
own capabilities (M4-04, `GET /v1/sessions/{session_id}/capabilities`). Domain
types remain independent of SQLx; the `mahfel-postgres` adapters (`Postgres*Store`) implement the
application ports and own SQL mapping. Ports return boxed futures and are dyn-compatible, so the
transport holds one `Arc<dyn Port>` per capability rather than a generic parameter per store
(ADR 0023). Authentication identities, permissions, participation records, and chat remain separate
later milestones.

Authorization is deny-by-default behind a capability seam in `mahfel-application`. One resolver maps a
verified `Principal` plus a room context to a `CapabilitySet`, and both REST and WebSocket evaluate
through it, so a control hidden in the UI and a request forged by hand are decided identically.
Capabilities replace a fixed role enum: global roles, per-room membership, and the service-principal
distinction all feed one resolution step, and a vertical maps its vocabulary onto capabilities
instead of adding roles (ADR 0020). The capabilities are:

| Capability | Meaning | Granted by |
| --- | --- | --- |
| `manage_system` | Realm, service principals, admin panel | Global `admin` role |
| `create_room` | Create a room | Any authenticated actor |
| `view_own_identity` | Read one's own profile | Any authenticated actor |
| `join` | Enter a session | Room membership, or a public room |
| `publish_media` | Send audio or video | Room membership |
| `share_screen` | Share a screen | Room membership |
| `mute_others` | Mute another participant | Room `manager`, or global `moderator` |
| `remove_participant` | Remove another participant | Room `manager`, or global `moderator` |
| `admit_participant` | Admit or reject a lobby waiter | Room `manager`, or global `moderator` |
| `manage_session` | Schedule, start, end, cancel, reconfigure | Room `manager` or global `admin` |

The domain owns the role-to-capability vocabulary: `RoleCapability::for_role` maps `manager` and
`member` to their grants, and the resolver names the transport-facing capability each one
corresponds to. Education maps onto this without new types — a teacher is a room `manager`, a
student is a room `member`, and attendance is the durable participation record. A caller may
introspect its own capabilities at `GET /v1/sessions/{session_id}/capabilities`; the response never
describes another participant's privileges. Global system administration is a capability like any
other, so there is no parallel role-based check.

The realtime gateway at `/v1/ws` is a bounded connection boundary: a versioned JSON envelope, an
authentication seam that denies by default, heartbeats, per-connection bounded queues with a write
timeout, and a process-local fan-out registry. It carries RTC signaling and session participation
frames.

### Media node registry (M4-08)

Each media process publishes its configured node ID, private gRPC URL, room limit, observed worker,
room and transport counts, and draining state every five seconds to the API's private gRPC registry
listener on port 50052. The browser-facing listener has no registry route. The bearer credential is
an HMAC-SHA256 token derived from an API-held root secret and that node ID, so a credential for one
node cannot register another node. Each deployment must provide a unique stable node ID, its own
derived token, and a directly reachable per-node gRPC URL. The checked-in token and root are
local-only fixtures; non-local deployments must override them with secrets.

The Redis adapter stores `media_nodes:record:<node-id>` as a hash with a 20-second TTL and
`media_nodes:live` as an expiring sorted index under the configured key prefix. One Redis script
updates the record, TTL, and index using Redis server time. Draining is sticky while the record
exists; a later heartbeat cannot accidentally clear it. On process shutdown, media publishes a
final draining heartbeat. A missing heartbeat lets both the record and index expire. Registry
reads exclude expired entries and return `Unavailable` on Redis failure or an oversized result,
so placement cannot treat an outage as an empty healthy registry.

The initial capacity limit is concurrent rooms per node. It is a configuration limit, not a claim
about participant or network capacity. Registry metrics report aggregate live and draining counts
and heartbeat outcomes with only bounded outcome labels. M4-09 uses the live-node reader for
weighted first placement and resolves later signaling through the session's durable node decision.
An assigned node that stops heartbeating makes its live session unavailable; it is not reassigned.

### Session participation and reconnect

Participation separates durable identity from transport:

- **Durable** (`mahfel.session_participants`, PostgreSQL). One row per `(session, actor)`, keyed by
  actor so a room or session has one owner column regardless of whether the actor is a user or a
  service principal. A partial unique index on `(session_id, actor_id) WHERE status = 'active'` makes
  "at most one active participant" a database invariant, so a reconnect, a second device, or a racing
  join cannot create a duplicate. Leaving is terminal, so history is append-only and the same actor
  may participate again later.
- **Ephemeral** (Redis connection leases and each API replica's `ConnectionRegistry`). A
  participant is present while at least one unexpired connection lease exists on any replica.
  Each lease has a unique connection token, is renewed while the socket runs, and expires after 45
  seconds if its replica dies. Redis is replaceable coordination state, never participant history.

The **multi-device policy** is: one participant identity per `(session, actor)`, and any number of
concurrent connections. Joining from a second device reuses the durable participant rather than
creating a new one, and the roster shows one entry. An operation that targets the participant (a mute
or a removal, M4-05) must therefore resolve across every connection that actor holds, not one
transport.

An explicit `leave` ends durable participation only after the last connection lease across all API
replicas is removed. An unexpected socket loss removes its own lease and closes the connected-time
interval only after global presence reaches zero; the participant row remains active so a reconnect
reuses its ID. A PostgreSQL advisory lock serializes join and last departure for each
`(session, actor)` across replicas. The lock uses a separate small connection pool so a join can
still use the ordinary stores while holding it.

Redis pub/sub wakes each replica when a lease changes. A versioned `session_roster` frame contains
the complete set of live participant IDs. Each replica also polls Redis every two seconds and sends
snapshots to its local sockets, so a dropped pub/sub subscriber or missed notice cannot leave a
roster stale indefinitely. The session's Redis lease set is bounded to 2,000 connections; local
outbound queues are bounded too. A Redis failure denies a new join or leave and disconnects a socket
whose lease cannot be renewed, rather than treating an unavailable global count as zero.

The connection generation guard remains **replica-local**. It orders repeated actions on one
connection in that process; it is not compared between replicas. Globally unique connection tokens
ensure an older cleanup removes only its own Redis lease. PostgreSQL still enforces one active
participant row per `(session, actor)` and owns durable history.

### Participation records

Connected time is durable, not derived from ephemeral presence. `mahfel.participation_intervals`
holds one row per connected stretch for a participant, keyed by `(participant_id, session_id,
actor_id)` and referencing the durable participant rather than a transport.

- **Intervals are append-only.** A correction does not overwrite the original; it marks the original
  `superseded_by` a replacement and records a `correction_reason` from a closed set, so the original
  measurement and the reason it changed both survive. Superseded intervals are excluded from totals,
  and superseding an open interval also closes it, so it cannot satisfy the one-open-interval rule.
- **At most one open interval per participant.** A partial unique index on `(participant_id) WHERE
  ended_at IS NULL` makes this a database invariant, so a duplicated connect or a racing join
  collides instead of recording two open stretches. Intervals for one participant therefore never
  overlap.
- **The reconnect policy is no grace after the last local device.** Losing one of several connections
  keeps the interval open. Losing the last closes it immediately, and a reconnect opens a new one
  against the same durable participant. Records fragment across a flaky connection, but there is no
  clock-skew window or expiry job to get wrong.
- **No interval outlives its session.** Ending a session sweeps every open interval in it, stopping
  the clock at the session end. Abrupt disconnects are covered because the WebSocket teardown closes
  the interval after the last local connection exits, not only after a graceful `leave`.

Queries are manager-scoped for a session summary and self-scoped for a participant's own history, so
a client can show its own connected time without seeing anyone else's. Corrections are manager-only
and audited. The capability model is the seam for these checks, and the transport endpoints are
`GET /v1/sessions/{session_id}/participation`,
`GET /v1/sessions/{session_id}/participants/{participant_id}/participation`, and
`POST /v1/participation/intervals/{interval_id}/correct`.

### Invite links

An invite link is a short, readable code that shares an entry path the way a meeting product does:
paste a code, get in. It is a revocable bearer capability (ADR 0021), not a second access system.

- **One namespace, one target.** `mahfel.invite_links` holds room links and session links in one
  table, so a code resolves deterministically. A link targets exactly one room or one session, which
  the domain models as a closed enum and the schema mirrors with a matching CHECK.
- **The code is a capability, not a secret.** Codes are `xxx-yyyy-zzz` — ten lowercase letters — and
  are stored plaintext because redemption looks them up directly. Protection comes from rate
  limiting, bounded attempts, non-enumerating errors, audit, and log hygiene. Codes never appear in
  structured logs, traces, or metric labels. Generation is rejection-sampled so every letter keeps
  equal probability, and uniqueness is the database's job, with a bounded retry that reports
  exhaustion rather than looping.
- **Redemption normalizes before lookup.** Case and separators are discarded, so a mixed-case paste
  or a code with spaces resolves to the same canonical value.
- **Expiry is derived, not stored.** A link's redeemability is decided by the clock against
  `expires_at` plus its status, so there is no sweep to run and no window where a stale status flag
  is trusted. `Revoked` and `Expired` are both rejected on the redemption path, not only in the UI.
- **Revocation is terminal; rotation is atomic.** Revoking twice is a conflict and never moves the
  revocation time. Rotation revokes the previous row and inserts the replacement in one transaction,
  so a reader observes exactly one live code. The previous row is retained, so the history shows what
  was retired and when.
- **Management is manager-gated.** Create, list, revoke, and rotate require management authority for
  the target, resolved from the caller's own membership. A session link is authorized through its
  room: the engine's model is that a session is managed by whoever manages its room.
- **Attempts are counted in Redis.** The limiter is a fixed window per principal and per source
  address, counted with `INCR` + `EXPIRE` behind the coordination port. It lives in Redis because the
  window must be shared across replicas and is replaceable coordination state, not a business fact.
  Identifiers reach the key space only as a digest, so the namespace cannot be read as a directory of
  actors or addresses.

`target_id` is deliberately not a foreign key: the target may be a room or a session, and link rows
outlive their target's operational lifetime. Resolution looks the target up and fails closed when it
is gone. Reaping stale links is intentionally deferred; the invariant is that a link with a missing
target is never redeemable.

#### Redemption

Redeeming a code resolves its target and, unless the caller already has access, queues a request for
approval. It never joins: resolution and admission are separate steps, and joining stays the join
path's job.

- **One failure for every unusable code.** A malformed code, an unknown one, an expired or revoked
  link, a target whose row is gone, and an archived room all return the same error. A caller cannot
  learn whether a code exists, whether it once existed, or whether its shape was even valid.
- **Attempts are counted before the code is read**, so a rate-limited attempt is indistinguishable
  from an invalid one and cannot be used to probe for free. Counters live in Redis, scoped per
  principal and per source address.
- **A source address is only trusted from the configured proxy.** Outside that, `X-Forwarded-For` is
  ignored and the socket peer is used, because trusting the header unconditionally would let any
  client forge a fresh budget.
- **The code travels in the request body**, never as a path segment, because paths are recorded by
  access logs and `Referer` headers. It is logged nowhere.
- **The room is always recorded** on a pending request, because authorization resolves against it for
  both scopes. A session id distinguishes the scopes: present means the request is for that one
  session, absent means it is for the room itself.

What approval grants depends on which it was. A **session** link grants attendance in that session
only and creates no durable membership, so the holder may be asked again at the next session. A
**room** link grants durable room membership with the `member` role, and later joins bypass the
confirmation step. A room approval needs no live session; a session approval does, because attending
a session that is not live is not meaningful.

Management endpoints are `POST`/`GET /v1/rooms/{room_id}/invite-links`,
`POST`/`GET /v1/sessions/{session_id}/invite-links`, `POST /v1/invite-links/{link_id}/revoke`, and
`POST /v1/invite-links/{link_id}/rotate`. Redemption and the confirmation flow that follows it are
M4-14 (TASK-130).

### Admission and the lobby

Joining is authorized first and admitted second. Authorization is unchanged — membership or public
visibility — and admission layers on top of it, so a caller who fails the join guard is still denied
and admission can never widen who is allowed.

- **Two axes, both inheritable.** A room carries an `admission` policy of `open` or `lobby`, and the
  existing `visibility`. A session carries an optional override of each; `NULL` in the database and
  `None` in the domain mean "inherit the room". A session therefore resolves to
  `session.effective_admission(room)` and `session.effective_visibility(room)`.
- **An override may only tighten.** A session can require approval that its room does not, or hide
  itself where its room is public, but it can never loosen either axis. That rule is enforced in the
  domain and is what makes an override safe to set: it cannot expose a room the room itself protects.
- **Waiting is a state, not a denial.** Under `lobby`, an authorized principal who is not already
  approved receives a pending admission request instead of a participant. The distinction matters:
  denied is terminal, waiting is resolvable by someone else.
- **A waiting principal holds nothing.** No participant row, no participation interval, and no
  `Join`/`PublishMedia` capability, because there is no participant identity to publish as. The
  restriction is structural rather than a UI convention.
- **Pending requests are durable, in PostgreSQL.** The waiting principal is looking at a waiting
  screen, so the request is a user-visible fact and does not belong in Redis. A partial unique index
  on `(session_id, actor_id) WHERE status = 'pending'` makes "at most one outstanding request per
  principal" a database invariant, so a reconnect while waiting keeps its place rather than queueing
  again. The waiting set is bounded per session, because any authorized caller could otherwise fill a
  manager's queue.
- **Decisions are capability-gated and audited.** Approving or rejecting requires
  `admit_participant`, which room managers and global moderators already hold. Approval is when
  access begins: it creates the participant and opens the participation interval, so the durable
  record starts at the decision rather than at the earlier request. Rejection terminates the waiting
  state and grants nothing.
- **Nothing outlives its session.** Ending a session retires every pending request alongside the
  participation sweep, and an expiry records no deciding principal, because no one made the call.
- **Requests carry an origin**, so an operator can tell an explicit join from an invite-link
  redemption (ADR 0021).

Endpoints are `GET`/`PUT /v1/rooms/{room_id}/admission`, `GET`/`PUT /v1/sessions/{session_id}/access`,
`GET /v1/sessions/{session_id}/lobby`, and
`POST /v1/sessions/{session_id}/lobby/{actor_id}/admit` and `/reject`. Over the WebSocket, a
deferred join is answered with a `join_waiting` frame instead of `joined`.

### Moderation

Moderation is capability-gated and generic: a vertical maps its vocabulary onto the capabilities
rather than adding engine roles, so education's teacher is a room `manager` and nothing about the
engine names a vertical.

- **Four rungs, each strictly stronger.** *Temporary mute* is a reset the target clears itself.
  *Permanent mute* holds until a caller with `mute_others` clears it. *Remove* ends the
  participation and permits rejoin. *Block* is a durable bar, so a blocked principal cannot rejoin
  while it is active. The rungs do not overlap: a caller who needs a mute that survives a rejoin
  wants a block.
- **Mute is durable state on the participation.** `muted_at`, `mute_kind`, and `muted_by` live on
  `mahfel.session_participants`, so a reconnect reuses the same row and the mute holds. It does
  **not** survive a rejoin: leaving is terminal, so a new join is a new participation with no mute.
  That is deliberate, and it is why block exists.
- **Blocks are independent durable facts.** Room-scoped and session-scoped blocks are separate rows.
  A room block denies every session in its room by enforcement, so nothing writes per-session rows
  for it, and lifting a room block can therefore never disturb an unrelated session block. A block
  carries an optional `expires_at`: `NULL` holds until explicitly lifted, which suits an education
  setting, while a timestamp lapses on its own, which suits a one-off meeting. Re-blocking an
  already-blocked principal extends the existing block rather than stacking a second one.
- **A block composes removal.** Barring someone who is currently publishing would not be a block, so
  blocking ends their participation and closes their producers.
- **Mute is enforced through media control by source.** A producer records what the SFU carries
  (`kind`: audio or video) and what it is for (`source`: microphone, camera, or screen). A screen
  share genuinely is video, so `kind` alone could not distinguish it from a camera. Muting pauses the
  named source and leaves the others running; unmuting resumes it without the target renegotiating
  transport. Pausing rather than closing is what keeps mute state and media state in one-to-one
  correspondence.
- **Publishing is gated by the capability that matches the source.** `publish_media` for a
  microphone or camera, `share_screen` for a screen. A room can therefore permit speaking without
  permitting a screen share, and a non-member in a public room — who holds `join` alone — cannot
  publish. A source is never authorization: it selects which producers an action applies to, so a
  mislabelled source can narrow an action but never widen one.
- **Withdrawing a capability reconciles running producers.** Removing a membership, or blocking,
  closes the producers whose source the principal may no longer publish. A demotion closes nothing,
  because a member keeps `publish_media` and `share_screen`; that is why this is a reconcile rather
  than a sweep. Enforcement failure is best-effort for the response and audited as degraded: the
  durable change has already happened, so failing the request would misreport it, but an operator has
  to know that a removed principal may still be publishing.
- **Identity is validated before enforcement.** A subject with no active participant fails `404`
  before any media call. The media plane holds no participant registry and must not gain one: a
  registry would make it authoritative for session membership, which would drift from PostgreSQL.
  It carries `participant_id` only as an ownership tag on transports, producers, and consumers.
- **Presence is at least one live connection.** A participant on two devices stays present when one
  closes, and `participant_left` is emitted only when the last connection goes. Moderation keys on
  the actor rather than a connection, so a mute or removal resolves across every device the
  participant is on.

Moderation actions are audited as structured `audit` events and propagate over typed WebSocket
frames: `participant_muted`, `participant_unmuted`, `participant_removed`, `principal_blocked`, and
`principal_unblocked`, driven by the `mute_participant`, `unmute_participant`, `remove_participant`,
`block_principal`, and `unblock_principal` client frames.

A room `manager` may change a member's role at `PATCH /v1/rooms/{room_id}/members/{user_id}`. The
last manager cannot be demoted, for the same reason it cannot be removed: the room would be left with
nobody who can manage it.

Known limitation: the connection registry is replica-local, so a participant with devices attached to
two replicas is counted once per replica. Cross-replica presence and targeting need the connection
hub (M5-03, ADR 0017) and are documented there rather than solved here.

## `mahfel-media`

The Rust media plane hosts Tokio, the mediasoup Rust bindings, native mediasoup workers, routers,
transports, producers, and consumers. It owns active-speaker signals and media statistics;
simulcast/SVC, bandwidth adaptation, and future recording/STT/AI media taps remain separate work.

It must not own authentication, courses, users, chat persistence, files, or LMS entities. Its gRPC
contract is private. The API remains responsible for session authorization and for mapping media
actor IDs to stable session participant IDs.

### Active speaker (M4-07)

Each room router holds a mediasoup `AudioLevelObserver` that watches microphone producers. The
observer reads the RTP audio-level header extension; senders that omit that extension cannot
contribute to the signal. A producer's samples are grouped by actor, so two devices holding one
participant identity occupy at most one speaker position. The initial observer interval is 200 ms,
its floor is -80 dBvo, and it returns at most eight producers. The tracker smooths levels with a
0.35 weight on the newest sample, enters after two samples at or above -65 dBvo, and exits after
three missing samples or samples at or below -75 dBvo. Near-equal speakers (within 3 dB) retain
their previous order. Ordinary changed snapshots are capped at four per second; silence and
explicit producer closure clear state immediately.

`WatchActiveSpeakers` is a private server-streaming RPC. It sends a reset frame and current
room-scoped snapshots when the API subscribes or when a subscriber falls behind. An empty actor
list means silence. Channels and WebSocket outbound queues are bounded. Each API replica
subscribes independently and fans events out only to its own WebSocket connections that have
joined the live session. The public `active_speakers` event carries an ordered complete list of
stable `participant_ids`, not audio levels or media producer IDs. A newly joined connection gets
the current snapshot. Waiting and merely authenticated connections receive no speaker event.
On media stream loss, the API clears the list, reconnects, and accepts fresh snapshots. Speaker
state is ephemeral and is never stored in PostgreSQL, Redis, or RabbitMQ.

WebSocket connections own the media transports they create. Explicit leave, EOF, heartbeat timeout,
write failure, and shutdown close only that connection's transports and their producers/consumers;
a second device's transports remain. The `media_active_speakers` gauge and
`media_speaker_stream_dropped_updates_total` counter have no room, actor, participant, or producer
labels. Ending a session clears the API speaker list and requests a media reset that closes
volatile room resources while retaining the router for a later session. A failed reset is logged.
Cross-replica presence and general WebSocket coordination remain with M5-03 (TASK-34).

## `mahfel-worker`

A private Rust process that consumes RabbitMQ jobs. Future responsibilities include recording
finalization, media processing, STT/TTS, LLM work, summaries, transcripts, thumbnails,
notifications, exports, and indexing. It has no public application API; only a private operations
listener.

## Infrastructure

- PostgreSQL is durable truth and hosts SQLx migrations.
- Redis provides ephemeral state and coordination through the `mahfel-redis` adapter (namespaced
  keys with TTLs); cross-replica signalling uses fire-and-forget pub/sub (ADR 0017); losing it must
  not destroy business data.
- RabbitMQ provides confirms, acknowledgements, redelivery, retry/DLQ, and competing consumers.
- S3-compatible storage holds files and media artifacts; RustFS implements the local interface.
- coturn supplies STUN/TURN traversal.
- Prometheus scrapes private operations ports and Grafana visualizes the data.
- Alertmanager provides the initial alert-routing boundary.
