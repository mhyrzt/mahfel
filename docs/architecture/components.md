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
win. Scheduling, starting, ending, and cancelling are manager-only (`/v1/rooms/{room_id}/sessions`
and `/v1/sessions/{session_id}/{start,end,cancel}`); service principals participate by being
managers of the rooms they own, and every lifecycle action writes a structured audit log. Reading a
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

### Session participation and reconnect

Participation separates durable identity from transport:

- **Durable** (`mahfel.session_participants`, PostgreSQL). One row per `(session, actor)`, keyed by
  actor so a room or session has one owner column regardless of whether the actor is a user or a
  service principal. A partial unique index on `(session_id, actor_id) WHERE status = 'active'` makes
  "at most one active participant" a database invariant, so a reconnect, a second device, or a racing
  join cannot create a duplicate. Leaving is terminal, so history is append-only and the same actor
  may participate again later.
- **Ephemeral** (in-process `ConnectionRegistry`). How many transports an actor currently holds, and
  which is newest, is volatile. A participant is present while at least one connection is live.

The **multi-device policy** is: one participant identity per `(session, actor)`, and any number of
concurrent connections. Joining from a second device reuses the durable participant rather than
creating a new one, and the roster shows one entry. An operation that targets the participant (a mute
or a removal, M4-05) must therefore resolve across every connection that actor holds, not one
transport.

#### Known limitations

These apply only when more than one API replica runs before the realtime fanout work
(M5-03, TASK-34) lands. On a single replica they do not arise.

- **The generation guard is replica-local.** Each connection carries a monotonic generation per
  `(session, actor)`, used to make a stale intent — a cleanup completing after a reconnect, or a
  delayed frame — a no-op. It is meaningful only within one API replica, because the registry is
  process-local and has no view of another replica's connections. Cross-replica ordering is not
  solved here; it needs Redis pub/sub (ADR 0017) and belongs to the connection-hub work in M5-03.
- **Roster fan-out is replica-local.** `participant_joined` / `participant_left` propagate only to
  connections on the same replica. A session spanning replicas produces partial roster updates until
  the M5-03 fanout hub exists.
- **Presence is per-replica.** "At least one live connection" is counted within one replica, so a
  participant with one device on replica A and one on replica B is counted once per replica.

These are consequences of the existing single-replica registry boundary, not of the participation
model. The durable identity, uniqueness, and history guarantees hold regardless of replica count.
M1-06 (TASK-7) delivered the Redis coordination adapter but never wired connection state onto it;
the fanout decision and the connection hub are M5-03.

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
- **The reconnect policy is no grace.** A disconnect closes the interval immediately, and a reconnect
  opens a new one against the same durable participant. Records fragment across a flaky connection,
  but the aggregate is exact and there is no clock-skew window or expiry job to get wrong.
- **No interval outlives its session.** Ending a session sweeps every open interval in it, stopping
  the clock at the session end. Abrupt disconnects are covered because the WebSocket teardown closes
  the interval on every exit path, not only a graceful `leave`.

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

Management endpoints are `POST`/`GET /v1/rooms/{room_id}/invite-links`,
`POST`/`GET /v1/sessions/{session_id}/invite-links`, `POST /v1/invite-links/{link_id}/revoke`, and
`POST /v1/invite-links/{link_id}/rotate`. Redemption and the confirmation flow that follows it are
M4-14 (TASK-130).

## `mahfel-media`

The Rust media plane hosts Tokio, the mediasoup Rust bindings, native mediasoup workers, routers,
transports, producers, and consumers. It will own simulcast/SVC, bandwidth adaptation, active
speaker signals, screenshare, media statistics, and future recording/STT/AI media taps.

It must not own authentication, courses, users, chat persistence, files, or LMS entities. Its gRPC
contract is private and control-oriented; only node information/statistics are implemented during
bootstrap. Lifecycle RPCs intentionally return `Unimplemented` until M3.

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
