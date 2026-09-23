# Testing strategy

Bootstrap keeps tooling small while establishing layers for later tests.

Rust unit tests cover pure library behavior. Integration tests cover database repositories,
RabbitMQ delivery semantics, and generated protocol compatibility. API tests exercise routers
without a network where possible and use disposable infrastructure for database behavior. Media
protocol tests verify serialization and explicit `Unimplemented` behavior before full RTC tests.

Frontend gates are strict TypeScript, ESLint, Vitest, Storybook, and production build. Vitest runs
unit and component tests in jsdom; Storybook documents and exercises shared UI components. Next.js
production builds are configured to package the application only; TypeScript validation is
intentionally performed by the separate `typecheck` recipe. Add focused component tests when
interactive behavior warrants them. Browser E2E arrives with usable cross-service flows; it should
cover real WebSocket and WebRTC behavior rather than mocking away the critical boundary.

Required commands:

```bash
cd backend
cargo fmt --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features
cargo nextest run --workspace --all-features
cargo test --workspace --all-features --doc

cd ../frontend
bun install
bun run lint
bun run typecheck
bun run test
bun run storybook:build
bun run build

cd ..
mkdocs build --strict
docker compose config
```

The REST contract is generated from the transport code with `utoipa` (ADR 0015). `mahfel-api`
serves the machine-readable document at `GET /v1/openapi.json`, and the interactive Scalar UI at
`/docs` on the private operations listener (ADR 0011). A unit test asserts that every REST route
template appears in the generated document, so a new route cannot be silently undocumented.
The document marks protected routes with bearer authentication, lists public routes explicitly,
and gives documented error responses the shared JSON error envelope.

Export both code-first documents to `specs/`, then generate frontend REST and WebSocket types
whenever either contract changes:

```bash
just api-docs-export
just frontend generate-api
```

The generated REST types live at `frontend/lib/api/generated/schema.ts`; WebSocket message types
live at `frontend/lib/websocket/generated/messages.ts`. The WebSocket contract is generated from
the client and server message enums with `asyncapi-rust` (ADR 0024). The export command writes
`specs/openapi.json` and `specs/asyncapi.json`. Its WebSocket operations use the API's perspective:
`receiveClientMessage` accepts client frames and `sendServerMessage` emits server frames. Each
payload includes the protocol version `v` and optional correlation `id`; clients should send
`v: 1`. Authenticate with a bearer token on the upgrade request or send an `auth` frame before
other client frames. The generated frontend types do not validate incoming frames at runtime.

An interactive Bruno collection for manual and end-to-end requests is planned under `bruno/`
(ADR 0016) and is not yet present; the directory holds a placeholder.

Redis coordination tests run against a disposable instance: set `MAHFEL_TEST_REDIS_URL` to an
isolated Redis, or `MAHFEL_TESTCONTAINERS=1` to start one. They cover TTL expiry, a pub/sub
round-trip, and that an unreachable backend maps to an `Unavailable` coordination error. The
outage test needs no server.

RTC load and resilience testing is a dedicated M14 deliverable, not a unit-test substitute.

Database integration tests use Testcontainers by default through `just backend test-database`. The harness
starts a disposable PostgreSQL container, creates a random disposable database, applies embedded
migrations twice to verify idempotent ledger behavior, asserts the expected schema, and removes the
container when the test exits. Docker must be available for this recipe. For an already-running
isolated PostgreSQL instance, set `MAHFEL_TEST_DATABASE_URL`; without either setting, the database
integration test is skipped. Never point it at a shared or production PostgreSQL cluster. Rust test
binaries run through `cargo-nextest`; doctests remain a separate `cargo test --doc` step because
nextest does not execute them.

OIDC tests use the Rust `testcontainers` 0.28 ecosystem with a `GenericImage` for the pinned
Keycloak image. The helper imports `deploy/keycloak/rastai-realm.json`, allocates and maps a dynamic
host port, and verifies OIDC discovery plus real access-token verification — issuer, audience, and
signature — against a token that carries the `mahfel-api` audience mapper; set
`MAHFEL_TESTCONTAINERS=1` to enable Docker-backed tests. Keycloak takes tens of seconds to start, so
the test allows a long startup timeout. A deterministic test with a local issuer and a fixed RSA key
covers the verifier's edge cases: expiry, wrong audience, wrong issuer, tampered signatures, and
JWKS key rotation. The API is a resource server: normal requests build the principal from the
`app_user_id` claim — injected by the realm's `mahfel_user_id` mapper — and never look up identities,
while `POST /v1/auth/provision` provisions the local user and writes the linking attribute (tested
end to end). No API login or callback route exists; browser login is `mahfel-web`'s responsibility
(ADR 0019).

Service principal and actor tests cover both credential families. API-key generation and hashing are
unit tested; the Postgres adapter is integration tested for create, authenticate, wrong key,
rotation invalidating the previous key, revocation, duplicate-name conflict, and a service-owned room
with its manager membership. API tests authenticate with a `mhf_` key, reject revoked keys and
principals with `401`, and prove the admin endpoints deny members and moderators while allowing an
administrator (ADR 0022). Session tests cover lifecycle transitions, the one-live-session-per-room
invariant under a concurrent start race, manager-only ownership, and the `409` mapping for an invalid
transition or a second live session.

Integration tests use Testcontainers and start disposable PostgreSQL, Redis, and Keycloak instances.
Run them with `MAHFEL_TESTCONTAINERS=1`, or per-suite with `just backend test-database` and the
`test-integration` recipe. The Postgres suites also accept an already-migrated local database through
`MAHFEL_TEST_DATABASE_URL`, and the Redis suite accepts `MAHFEL_TEST_REDIS_URL`, for environments where
the Docker socket is unavailable to Testcontainers; without either setting the suite prints a skip
notice and returns success, so do not read a green run as proof the database path executed.

When a suite supports both, prefer the Testcontainers path: it provisions a throwaway instance and
cannot touch shared state. Note that PostgreSQL logs `database system is ready to accept connections`
before it can complete a client authentication handshake, so the Postgres suites probe with a real
connection (`wait_until_ready`) rather than trusting that log line or sleeping a fixed amount.

A pre-existing failure in `mahfel-auth`'s `keycloak_tokens_verify_and_reject_tampering` is unrelated
to database or session work: the test mutates the final base64 character of a signature and expects
`InvalidSignature`, which does not hold against the pinned Keycloak image.

SQL migrations use SQLFluff with the PostgreSQL dialect. `just backend sql-lint` checks migration
files and `just backend sql-fmt` applies SQLFluff formatting. The backend `lint` recipe includes the
SQL lint check. SQLFluff is expected to be available in the developer environment or CI image.
Protobuf contracts use Buf; run `just proto fmt-check` and `just proto lint` before changing the
generated tonic boundary.
