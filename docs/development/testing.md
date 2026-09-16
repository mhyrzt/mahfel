# Testing strategy

Bootstrap keeps tooling small while establishing layers for later tests.

Rust unit tests cover pure library behavior. Integration tests cover database repositories,
RabbitMQ delivery semantics, and generated protocol compatibility. API tests exercise routers
without a network where possible and use disposable infrastructure for database behavior. Media
protocol tests verify serialization and explicit `Unimplemented` behavior before full RTC tests.

Frontend gates are strict TypeScript, ESLint, and production build. Next.js production builds are
configured to package the application only; TypeScript validation is intentionally performed by the
separate `typecheck` recipe. Add focused component tests when interactive behavior warrants them.
Browser E2E arrives with usable cross-service flows; it should cover real WebSocket and WebRTC
behavior rather than mocking away the critical boundary.

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
bun run build

cd ..
mkdocs build --strict
docker compose config
```

RTC load and resilience testing is a dedicated M14 deliverable, not a unit-test substitute.

Database integration tests use Testcontainers by default through `just test-database`. The harness
starts a disposable PostgreSQL container, creates a random disposable database, applies embedded
migrations twice to verify idempotent ledger behavior, asserts the expected schema, and removes the
container when the test exits. Docker must be available for this recipe. For an already-running
isolated PostgreSQL instance, set `MAHFEL_TEST_DATABASE_URL`; without either setting, the database
integration test is skipped. Never point it at a shared or production PostgreSQL cluster. Rust test
binaries run through `cargo-nextest`; doctests remain a separate `cargo test --doc` step because
nextest does not execute them.

SQL migrations use SQLFluff with the PostgreSQL dialect. `just backend sql-lint` checks migration
files and `just backend sql-fmt` applies SQLFluff formatting. The backend `lint` recipe includes the
SQL lint check. SQLFluff is expected to be available in the developer environment or CI image.
