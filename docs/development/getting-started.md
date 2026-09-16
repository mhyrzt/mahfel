# Getting started

## Prerequisites

Install Docker Engine with Compose v2 separately, then install each remaining tool below. Native
mediasoup dependencies come first because they are required when compiling the Rust media service
on the host. Commands target Fedora unless a platform-specific command is shown.

### Native mediasoup toolchain

These compilers and libraries build the native mediasoup worker during host-side Rust builds.

```bash
sudo dnf install gcc-c++ clang cmake make ninja-build pkgconf-pkg-config \
  libstdc++-static
```

### Bun

Bun is Mahfel's frontend package manager, script runner, and local Next.js runtime. Follow the
installation instructions in the [oven-sh/bun GitHub repository](https://github.com/oven-sh/bun).

### uv

uv provides fast, isolated installation of Python CLI tools without a project-wide Python
environment. Follow the [astral-sh/uv GitHub repository](https://github.com/astral-sh/uv), then run:

```bash
uv tool install 'sqlfluff[rs]'
uv tool install mkdocs
uv tool install mkdocs-material
```

SQLFluff checks and formats PostgreSQL migrations; MkDocs and Material build this documentation site.

### Rust

Rust compiles the API, media, and worker services. Install rustup from the
[rust-lang/rustup GitHub repository](https://github.com/rust-lang/rustup), then select the pinned
toolchain:

```bash
rustup toolchain install 1.95.0
rustup component add rustfmt clippy --toolchain 1.95.0
```

### Just

Just is the repository task runner used by the root, backend, frontend, and protobuf workflows.
Install it from the [casey/just GitHub repository](https://github.com/casey/just):

```bash
cargo install just --locked
```

### cargo-nextest

Nextest runs Rust tests with better isolation and reporting than the default test harness. Install
it from the [nextest-rs/nextest GitHub repository](https://github.com/nextest-rs/nextest):

```bash
cargo install cargo-nextest --locked
```

### Buf

Buf formats and lints the shared protobuf contract. Install it from the
[bufbuild/buf GitHub repository](https://github.com/bufbuild/buf):

```bash
brew install bufbuild/buf/buf
```

The repository pins the Rust toolchain in `backend/rust-toolchain.toml` and Bun in
`frontend/package.json`; the commands above install compatible versions. Docker builds include
their own native toolchain. A Docker daemon is still required by Testcontainers integration tests.

## First checkout

```bash
cp .env.example .env
just frontend install
cp frontend/.env.local.example frontend/.env.local
just compose-up
```

`just compose-up` starts all three Compose profiles (`mahfel`, `infrastructure`, and
`observability`), builds application images, and installs/trusts Caddy's local CA. Browse through
`https://mahfel.localhost`; do not bypass certificate warnings.

## Host-run development

To iterate on Rust or Next.js without rebuilding application images, start only infrastructure:

```bash
just compose-up infrastructure
just frontend dev
just backend dev-api
just backend dev-media
just backend dev-worker
```

Host processes use `backend/config/local.toml` and environment overrides. The frontend uses
`frontend/.env.local`. Use direct loopback ports for this mode, or configure the Caddy upstream
variables documented in [local deployment](../deployment/local.md) when HTTPS domains are needed.

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
make repeated startup safe. `just backend test-database` starts a disposable PostgreSQL Testcontainer and
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
