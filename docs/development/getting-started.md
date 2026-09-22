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
just bootstrap
just frontend install
just compose-up
```

`just bootstrap` creates `.env` and `frontend/.env.local` only when they do not already exist,
starts only Caddy without building application images, and
installs/trusts Caddy's local CA. `just compose-up` then starts all three Compose profiles (`mahfel`, `infrastructure`, and
`observability`) and builds application images. Local OIDC is available at
`https://auth.mahfel.localhost/realms/rastai`; the same Keycloak provider is used in production. Browse through
`https://mahfel.localhost`; do not bypass certificate warnings.

To start infrastructure, use `just compose-up infrastructure`. To start only Caddy for certificate
or host-upstream work, use `just caddy`.

## Minikube

```bash
just minikube init
just minikube bootstrap
```

Initialize the cluster once. `just minikube bootstrap` builds and deploys the images, then keeps
local HTTPS forwarding open until you stop the command. Host ports 80 and 443 must be free.
`just minikube up` exports the Minikube Caddy CA to `.local/minikube/root.crt` and installs it in
the host trust store; this step may prompt for `sudo`. The forwarding step also prompts for `sudo`
to bind host ports 80 and 443. Open `https://mahfel.localhost` when forwarding starts.
Run `just minikube cert` to export and trust the CA again if the Minikube Caddy certificate changes.
Restart open browsers after trusting it.
If `sudo` cannot prompt during `just minikube up`, run `just minikube cert` from an interactive
terminal to finish host trust after the cluster is ready.
On a first start, pulling the infrastructure and initialization images can take several minutes.
If a rollout or job times out, `just minikube up` prints the affected pod status, recent events,
and available container logs.

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

The local realm fixture seeds an administrator; bootstrapping a service principal locally is to log
in through `mahfel-web` as that administrator, let `POST /v1/auth/provision` link the local user, then
call `POST /v1/admin/service-principals` with the resulting access token. The raw API key is returned
once and cannot be recovered (ADR 0022).

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

Migration `0002` was amended in place to rename `classroom_sessions` to `sessions` and add the
one-live-session-per-room index, because the schema has not been deployed beyond local development.
SQLx records a checksum per applied migration, so an already-migrated local database reports a
checksum mismatch for `0002`; reset the local database (`just compose-clean`, or drop and recreate
the development database) and run `just backend migrate` again rather than editing the migration
ledger.

Common component gates are recipes under `just backend <recipe>` and `just frontend <recipe>`;
`just docs` builds documentation, `just clean` removes generated local artifacts, and
`just compose-clean` resets Docker volumes. The root `justfile` keeps only these component proxies. Use
`just backend <recipe>` or `just frontend <recipe>` for a focused workflow; running `just` inside
either component lists its local recipes. The frontend uses Bun exclusively. Rust uses edition 2024
and the pinned toolchain in `backend/rust-toolchain.toml`.

On Fedora-like hosts, the mediasoup crate requires the static libstdc++ development package because
it links the native worker library statically. The container builder includes the corresponding
Debian development package. The backend image has `api`, `media`, and `worker` build targets; use
`docker build --target media -f backend/Dockerfile .` when the host library is unavailable.
