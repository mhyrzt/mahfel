# Components

## `mahfel-web`

An independently deployed Next.js App Router application built with Bun, TypeScript, Tailwind CSS,
and shadcn/ui. It eventually owns all user and administration interfaces, device control,
`mediasoup-client`, and Yjs integration. It is the browser OIDC client (NextAuth with Keycloak) and
exposes a narrow BFF for authenticated API calls (ADR 0019). It does not contain authoritative
backend logic.

## `mahfel-api`

The Rust control plane and authoritative application service. It owns REST, WebSocket signaling,
identity integration, authorization, users, rooms and sessions, attendance, persistent chat, file
metadata and presigning, collaborative-state orchestration, media-node scheduling, PostgreSQL
persistence, Redis coordination, RabbitMQ publishing, and the gRPC media client.

It exposes the versioned `/v1` control-plane API (status, identity, and room endpoints) plus a
WebSocket gateway at `/v1/ws`. The REST contract is generated from the transport code and served as
OpenAPI at `/v1/openapi.json`, with its interactive UI on the private operations listener
(ADR 0015). Higher-level product surfaces remain deliberately deferred.

OIDC is provided by Keycloak for both local development and production (`auth.mahfel.localhost`
locally). The browser authenticates through `mahfel-web` (NextAuth with the Keycloak provider) and a
narrow BFF (ADR 0019). The API is a resource server: it builds the principal from the `app_user_id`
claim without an identity lookup, and `POST /v1/auth/provision` provisions the local user and links
its id onto the Keycloak identity. The API also manages the realm through the Keycloak Admin API
using the client-credentials grant of a dedicated service account rather than an interactive
administrator account. Identity linking, role mapping, and logout remain M2 implementation concerns;
the local realm fixture is described in `deploy/keycloak/README.md`.

The first control-plane persistence boundary is intentionally small: users, rooms, and classroom
sessions have UUID identifiers, explicit lifecycle statuses, and UTC timestamps in PostgreSQL.
A room is a durable container with `public` or `invite_only` visibility and room-scoped memberships
whose roles are `manager` and `member`. A room's identity is its id alone, so titles need not be
unique. Domain types remain independent of SQLx; the `mahfel-postgres` adapters (`Postgres*Store`)
implement the application ports and own SQL mapping. Authentication identities, permissions,
attendance, and chat remain separate later milestones.

Authorization is deny-by-default behind an `Authorizer` seam in `mahfel-application`: the transport
attaches a `Principal`, the policy decides coarse actions, and room-scoped membership checks live in
the room use cases. The interim policy grants any authenticated principal and is replaced by the
classroom role model (M2-07).

The realtime gateway at `/v1/ws` is a bounded connection boundary: a versioned JSON envelope, an
authentication seam that denies by default, heartbeats, per-connection bounded queues with a write
timeout, and a process-local fan-out registry. It carries test events only; signaling semantics,
cross-replica delivery, and presence are later work.

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
