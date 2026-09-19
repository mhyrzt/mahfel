# Components

## `mahfel-web`

An independently deployed Next.js App Router application built with Bun, TypeScript, Tailwind CSS,
and shadcn/ui. It eventually owns all user and administration interfaces, device control,
`mediasoup-client`, and Yjs integration. It does not contain authoritative backend logic.

## `mahfel-api`

The Rust control plane and authoritative application service. It owns REST, WebSocket signaling,
identity integration, authorization, users, rooms and sessions, attendance, persistent chat, file
metadata and presigning, collaborative-state orchestration, media-node scheduling, PostgreSQL
persistence, Redis coordination, RabbitMQ publishing, and the gRPC media client.

The service currently exposes only a status endpoint and infrastructure health. Product routes are
deliberately deferred.

For local development, OIDC is provided by Dex (`auth.mahfel.localhost`) with SQLite-backed state.
The API is intended to consume standard issuer/discovery, authorization-code, token, and userinfo
contracts rather than Dex-specific management endpoints. Production continues to use Keycloak by
configuration; identity linking, claim normalization, role mapping, logout, and user migration must
be validated as a separate deployment concern. Dex is never an additional production proxy or
federation layer.

The first control-plane persistence boundary is intentionally small: users, rooms, and classroom
sessions have UUID identifiers, explicit lifecycle statuses, and UTC timestamps in PostgreSQL.
A room is a durable container with `public` or `invite_only` visibility and room-scoped memberships
whose roles are `manager` and `member`. A room's identity is its id alone, so titles need not be
unique. Domain types remain independent of SQLx; the `mahfel-postgres` adapters (`Postgres*Store`)
implement the application ports and own SQL mapping. Authentication identities, permissions,
attendance, and chat remain separate later milestones.

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
- Redis provides ephemeral state and coordination; losing it must not destroy business data.
- RabbitMQ provides confirms, acknowledgements, redelivery, retry/DLQ, and competing consumers.
- S3-compatible storage holds files and media artifacts; RustFS implements the local interface.
- coturn supplies STUN/TURN traversal.
- Prometheus scrapes private operations ports and Grafana visualizes the data.
- Alertmanager provides the initial alert-routing boundary.
