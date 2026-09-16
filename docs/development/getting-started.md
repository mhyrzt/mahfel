# Getting started

```bash
cp .env.example .env
just frontend install
just compose-up
```

For host development, infrastructure may be started separately and each application launched with
`just frontend dev`, `just backend dev-api`, `just backend dev-media`, or `just backend dev-worker`. Host Rust processes load
`backend/config/default.toml`, the selected `backend/config/{profile}.toml` profile, optional
`backend/config/local.toml` overrides, then `MAHFEL_` environment overrides. Docker uses the
`docker` profile; host `just backend dev-*` recipes use `local`. Nested keys use double underscores, for example
`MAHFEL_DATABASE__HOST=127.0.0.1` and `MAHFEL_DATABASE__PORT=5432`. PostgreSQL connection settings
are separate `host`, `port`, `user`, `password`, and `name` values; application configuration does
not require a constructed connection URL.

## Database lifecycle

Pool bounds and timeouts live under `[database]` and can be overridden with variables such as
`MAHFEL_DATABASE__MAX_CONNECTIONS`. Production deployments should run the forward-only migration
command once before rolling application replicas:

```bash
just backend migrate
# or, using the built image:
docker compose run --rm mahfel-api migrate
```

`migrate_on_start` defaults to false to avoid coupling every production replica startup to schema
changes. Local Compose enables it for convenience; SQLx migration locking and its migration ledger
make repeated startup safe. `just test-database` starts a disposable PostgreSQL Testcontainer and
verifies the embedded migrations against a uniquely named database, then removes the container.

Common component gates are recipes under `just backend <recipe>` and `just frontend <recipe>`;
`just docs` builds documentation. The root `justfile` keeps only these component proxies. Use
`just backend <recipe>` or `just frontend <recipe>` for a focused workflow; running `just` inside
either component lists its local recipes. The frontend uses Bun exclusively. Rust uses edition 2024
and the pinned toolchain in `backend/rust-toolchain.toml`.

On Fedora-like hosts, the mediasoup crate requires the static libstdc++ development package because
it links the native worker library statically. The container builder includes the corresponding
Debian development package. The backend image has `api`, `media`, and `worker` build targets; use
`docker build --target media -f backend/Dockerfile .` when the host library is unavailable.
