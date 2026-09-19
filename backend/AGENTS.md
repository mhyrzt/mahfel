# Mahfel Backend — Agent Instructions

These instructions apply to everything under `backend/`.

Also follow the repository-level `../AGENTS.md`.

Mahfel's backend is a Rust 2024 workspace with three runtime executables:

- `mahfel-api`
- `mahfel-media`
- `mahfel-worker`

The backend is modular, but these are the intended deployment units. Do not create new deployable services without an explicit architectural decision.

## Toolchain

Use the pinned Rust toolchain from:

```text
backend/rust-toolchain.toml
```

Use Cargo through the repository's `just` commands where practical.

Do not weaken workspace lints to make code compile.

Production code must continue respecting the workspace constraints against:

- `unsafe`
- `unwrap`
- `todo!`
- `unimplemented!`
- `dbg!`

Tests may use explicit `expect(...)` when the failure message improves diagnostics.

## Backend Architecture

The primary dependency direction is:

```text
domain
  ↑
application
  ↑
adapters / transports / composition roots
```

More concretely:

```text
mahfel-domain
      ↑
mahfel-application
      ↑
 ┌────┼───────────────────────────────┐
 │    │                               │
postgres / messaging / media-client   runtime binaries
```

Infrastructure details must point inward through application ports rather than leaking into domain/application logic.

### `mahfel-domain`

The domain crate contains durable business concepts, invariants, value objects, identifiers, and state transitions.

It must remain independent of infrastructure frameworks.

Do not introduce dependencies on:

- Axum
- SQLx
- Tonic
- Lapin
- Redis clients
- HTTP types
- database rows
- transport DTOs

Put validation and state-transition invariants in domain types when they are genuinely domain rules.

Prefer invalid states being difficult or impossible to construct.

Do not turn `mahfel-domain` into a dumping ground for unrelated shared structs.

### `mahfel-application`

The application crate owns use cases and outbound port contracts.

Application code may orchestrate domain behavior and abstract dependencies.

It must not depend directly on infrastructure clients such as SQLx, Lapin, Tonic, or Axum.

If a use case needs persistence, messaging, media control, or another external capability, express the requirement through a narrow application port.

Prefer feature/vertical-slice organization such as:

```text
application/src/
  rooms/
  sessions/
  users/
  ports/
```

Do not create generic `services`, `helpers`, or `utils` modules when a domain- or feature-specific home is clearer.

### Infrastructure adapters

Infrastructure crates implement application ports.

Examples:

- `mahfel-postgres`
- `mahfel-messaging`
- `mahfel-media-client`

Keep concrete infrastructure errors and types at the adapter boundary.

Convert them into application-facing errors deliberately.

Do not expose `sqlx::Error`, `lapin` types, Tonic clients, or equivalent infrastructure types through application or domain APIs.

### Runtime binaries

Runtime binaries are composition roots.

They may:

- load configuration;
- initialize telemetry;
- construct infrastructure clients;
- wire application ports to adapters;
- build HTTP/gRPC servers;
- coordinate lifecycle and graceful shutdown.

They should not accumulate core business rules.

If significant business logic appears in `bins/*/main.rs`, move it into the appropriate library boundary.

## `mahfel-api`

`mahfel-api` is the authoritative control plane.

It owns browser-facing:

- REST;
- WebSocket signaling;
- authentication integration;
- authorization;
- durable application operations;
- coordination with media and workers.

The API may call `mahfel-media` through the private media client.

Do not expose mediasoup or internal media-node implementation details through public handlers unless they are intentionally mapped into a stable application contract.

### HTTP APIs

Public product APIs live under:

```text
/v1
```

The API is served on its own host, so the version prefix is not nested under `/api`.

Maintain stable, explicit request and response contracts.

Do not return raw internal errors to clients.

Map errors into deliberate HTTP semantics.

Validation failures and known domain failures must not become generic HTTP 500 responses.

Request IDs should remain available for correlation.

Authorization checks belong at a deliberate boundary and should default to deny.

### WebSockets

Treat every WebSocket connection as bounded state.

Use:

- bounded queues;
- explicit message/envelope versions;
- heartbeat/liveness handling;
- graceful disconnect;
- backpressure behavior.

Do not create unbounded channels for client-controlled traffic.

Do not assume a client will remain connected to the same API replica forever.

## `mahfel-media`

`mahfel-media` is the realtime media plane.

It owns:

- mediasoup workers;
- routers;
- WebRTC servers;
- transports;
- producers;
- consumers;
- realtime media statistics.

It must not become authoritative for:

- users;
- authentication;
- classroom membership;
- chat;
- file metadata;
- LMS data;
- durable application state.

The API/media boundary is private gRPC.

Browsers must never call this gRPC interface directly.

Keep mediasoup-specific types behind the RTC/media boundary.

When implementing lifecycle operations, design them to be idempotent where retries are expected.

Media resources require explicit cleanup. Test repeated create/delete/disconnect paths for leaks.

## `mahfel-worker`

`mahfel-worker` is a private asynchronous worker.

It must not expose a public product API.

Job processing must be designed for at-least-once delivery.

Handlers must assume duplicate delivery can happen.

Long-term worker behavior should use:

- manual acknowledgements;
- bounded concurrency;
- idempotency;
- bounded retry;
- dead-letter handling;
- graceful draining.

Do not acknowledge a job before the durable effect that defines successful completion.

## PostgreSQL

PostgreSQL is the durable source of truth.

Use SQLx.

Prefer explicit SQL and typed mapping over hiding persistence behind a heavyweight ORM.

### Migrations

Schema changes belong in reviewed forward migrations under:

```text
backend/migrations/
```

Never perform hidden schema mutation from normal application startup logic.

Keep domain validation and database constraints aligned where practical.

Database constraints are defense in depth; they do not replace domain validation.

Do not edit an already-deployed migration to represent a new schema transition. Add a new migration.

### Repositories and stores

Reuse existing application ports and PostgreSQL adapters before creating another abstraction.

Avoid parallel repository layers representing the same responsibility.

Queries should select only required columns.

Use transactions when a business operation requires atomic durable effects.

Never construct SQL by interpolating untrusted values.

When dynamic SQL is genuinely required, constrain and justify the dynamic portion.

## Redis

Redis is ephemeral coordination state only.

Suitable examples include:

- presence;
- TTL-backed leases;
- node heartbeats;
- room placement;
- cross-replica coordination.

If losing Redis would permanently destroy a user-visible business fact, the design is wrong.

Every Redis key family should have explicit namespace and lifecycle semantics.

Use TTLs where stale coordination state would otherwise remain indefinitely.

## RabbitMQ

RabbitMQ is a delivery mechanism, not business truth.

Publishers should use confirms for durable job/event flows.

Consumers must be safe under redelivery.

Do not design a handler that assumes "exactly once" delivery.

Job/event schemas should be explicit and versionable.

Do not place secrets or unnecessarily sensitive data into event payloads.

## gRPC / Protobuf

The media protocol source lives in `../proto`.

Generated protocol types are transport contracts, not domain entities.

Convert between protocol types and internal domain/application types at the boundary.

Use deadlines/timeouts for service-to-service calls where appropriate.

Map gRPC failures intentionally rather than converting every status into an opaque internal error.

Do not leak gRPC ports through public ingress.

## Configuration

Configuration is layered:

```text
default.toml
→ profile TOML
→ optional local.toml
→ MAHFEL_* environment overrides
```

Nested environment keys use double underscores.

Example:

```text
MAHFEL_DATABASE__HOST
```

Keep non-secret defaults in configuration files.

Secrets and environment-specific production values belong in environment variables or the deployment secret store.

Do not add a second unrelated configuration mechanism.

## Errors

Library crates should expose typed errors using `thiserror`.

Runtime binaries may use `anyhow` to attach composition/startup context.

Errors should preserve enough context for operators without leaking secrets.

Prefer semantic variants:

```text
NotFound
Conflict
InvalidInput
Unauthorized
Unavailable
```

over forcing callers to inspect error strings.

Avoid stringly typed control flow.

## Async Rust

Do not block Tokio executor threads with blocking CPU or synchronous I/O.

Use `spawn_blocking` or an appropriate dedicated execution path for genuinely blocking work.

Keep lock scope small.

Never hold an async lock across slow network or database operations unless the design specifically requires it.

Prefer cancellation-aware loops and graceful shutdown.

Bound queues and concurrency when inputs can be externally driven.

## Observability

Use `tracing` rather than `println!`.

Preserve structured fields.

Do not log secrets, authorization material, raw credentials, or sensitive payloads.

Prometheus labels must have bounded cardinality.

Operations endpoints remain on their dedicated private listeners:

```text
mahfel-api     :9101
mahfel-media   :9102
mahfel-worker  :9103
```

Keep liveness process-local.

Use readiness for critical ability to perform work.

Do not make noncritical dependency degradation trigger restart loops.

## Testing

Choose the smallest test layer capable of proving the behavior.

### Unit tests

Use unit tests for:

- domain invariants;
- state transitions;
- validation;
- pure application behavior;
- deterministic conversion logic.

### Integration tests

Use integration tests for real infrastructure boundaries such as:

- PostgreSQL;
- OIDC providers;
- RabbitMQ;
- protocol behavior.

Use disposable infrastructure through Testcontainers where the project already supports it.

Never point destructive integration tests at shared or production infrastructure.

Do not replace an integration test with a mock when the purpose of the test is to verify interoperability with the real dependency.

### Regression tests

When fixing a bug, add a regression test when the behavior can be reproduced reliably.

The regression test should fail before the fix and pass afterward.

## Validation

During development, run focused tests for the changed crate or behavior first.

Before completing normal backend changes, use the relevant subset of:

```bash
just backend fmt
just backend lint
just backend test
just backend build
```

When changing SQL migrations, also ensure SQL linting passes.

When changing protobuf contracts:

```bash
just proto fmt-check
just proto lint
```

Do not claim integration tests passed if Docker-backed tests were skipped because their enabling environment was unavailable.

If a command reveals a pre-existing failure unrelated to the task, distinguish that clearly from failures introduced by the change.

## Code Quality

Prefer straightforward Rust over clever Rust.

Use strong types when they encode meaningful invariants.

Do not introduce traits solely for hypothetical future implementations.

Introduce abstraction when there is a real boundary, substitution need, or testing seam.

Keep public APIs small.

Prefer ownership and borrowing that make lifecycle clear over unnecessary cloning.

Do not optimize before measurement, but avoid obviously unbounded behavior.

Make concurrency limits, timeout behavior, retries, and resource ownership explicit for networked code.

## Before Finishing Backend Work

Check that:

- business rules are in the correct layer;
- infrastructure does not leak into domain/application code;
- new durable state is stored in PostgreSQL;
- ephemeral state is not accidentally authoritative;
- retries cannot duplicate durable effects;
- queues/concurrency are bounded where necessary;
- errors map cleanly across boundaries;
- telemetry does not introduce high-cardinality labels;
- relevant tests pass;
- documentation or ADRs are updated when behavior or architecture changed.
