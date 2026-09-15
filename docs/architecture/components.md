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

The first control-plane persistence boundary is intentionally small: users, rooms, and classroom
sessions have UUID identifiers, explicit lifecycle statuses, and UTC timestamps in PostgreSQL.
Domain types remain independent of SQLx; repository adapters in `mahfel-database` own SQL mapping.
Authentication identities, memberships, permissions, and attendance are separate later
milestones and are not implied by these foundational records.

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
- S3-compatible storage holds files and media artifacts; MinIO implements the local interface.
- coturn supplies STUN/TURN traversal.
- Prometheus scrapes private operations ports and Grafana visualizes the data.
- Alertmanager provides the initial alert-routing boundary.
