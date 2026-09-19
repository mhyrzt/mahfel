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

OIDC integration tests use the existing Rust `testcontainers` 0.28 ecosystem with a `GenericImage`
for the pinned Keycloak image. The helper imports `deploy/keycloak/rastai-realm.json`, allocates and
maps a dynamic host port, and verifies OIDC discovery and the `mahfel-admin` client-credentials
service account without assuming `localhost:8080`; set `MAHFEL_TESTCONTAINERS=1` to enable
Docker-backed tests. Keycloak takes tens of seconds to start, so the test allows a long startup
timeout. `testcontainers-modules` currently targets an incompatible dependency, so it is not
introduced just to wrap this small fixture. The bootstrap currently has no API login or callback
route, so authorization-code exchange and session assertions remain M2 work rather than being faked
by the provider fixture.

SQL migrations use SQLFluff with the PostgreSQL dialect. `just backend sql-lint` checks migration
files and `just backend sql-fmt` applies SQLFluff formatting. The backend `lint` recipe includes the
SQL lint check. SQLFluff is expected to be available in the developer environment or CI image.
Protobuf contracts use Buf; run `just proto fmt-check` and `just proto lint` before changing the
generated tonic boundary.
