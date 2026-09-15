# Getting started

```bash
cp .env.example .env
just bootstrap
just compose-up
```

For host development, infrastructure may be started separately and each application launched with
`just dev-web`, `just dev-api`, `just dev-media`, or `just dev-worker`. Host Rust processes load
`backend/config/default.toml`, optional `backend/config/local.toml`, then `MAHFEL_` environment
overrides. Nested keys use double underscores, for example
`MAHFEL_API__DATABASE_URL=postgres://...`.

Common gates are `just fmt`, `just lint`, `just test`, `just build`, and `just docs`. The frontend
uses Bun exclusively. Rust uses edition 2024 and the pinned toolchain in
`backend/rust-toolchain.toml`.

On Fedora-like hosts, the mediasoup crate requires the static libstdc++ development package because
it links the native worker library statically. The container builder includes the corresponding
Debian development package. Generic backend work can be checked in the production builder when the
host library is unavailable.
