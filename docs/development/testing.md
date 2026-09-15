# Testing strategy

Bootstrap keeps tooling small while establishing layers for later tests.

Rust unit tests cover pure library behavior. Integration tests cover database repositories,
RabbitMQ delivery semantics, and generated protocol compatibility. API tests exercise routers
without a network where possible and use disposable infrastructure for database behavior. Media
protocol tests verify serialization and explicit `Unimplemented` behavior before full RTC tests.

Frontend gates are strict TypeScript, ESLint, and production build. Add focused component tests when
interactive behavior warrants them. Browser E2E arrives with usable cross-service flows; it should
cover real WebSocket and WebRTC behavior rather than mocking away the critical boundary.

Required commands:

```bash
cd backend
cargo fmt --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features
cargo test --workspace

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

Database integration tests use `MAHFEL_TEST_DATABASE_URL` as an administrative connection. The
harness creates a random disposable database, applies embedded migrations twice to verify
idempotent ledger behavior, asserts the expected schema, and removes the database. Use
`just test-database`; do not point the variable at a shared or production PostgreSQL cluster.
