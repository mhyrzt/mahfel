# Local deployment

## Prerequisites

- Docker Engine with Compose v2
- Bun for host frontend work
- Rust 1.95 with rustfmt and Clippy for host backend work
- `just`, `cargo-nextest`, and MkDocs Material for convenience, tests, and documentation
- C++ toolchain, Python/pip, CMake, and pkg-config for native mediasoup builds

Initialize the local environment once with:

```bash
just bootstrap
```

`just bootstrap` creates `.env` and `frontend/.env.local` without overwriting existing files, starts
only Caddy without building application images, and runs `just trust-local-ca`.
The trust step uses `sudo` to install the generated root certificate into Fedora/RHEL,
Debian/Ubuntu, or macOS system trust. This is mandatory: do not bypass browser certificate
warnings. Restart open browsers after the first trust operation. Caddy keeps its CA in the
persistent `caddy-data` volume, so subsequent runs retain the same trusted root unless that volume
is intentionally deleted.

After initialization, start the complete local stack with:

```bash
just compose-up
```

Compose services are grouped into three stack profiles: `mahfel` (the four buildable application services),
`infrastructure` (Caddy, PostgreSQL, Redis, RabbitMQ, RustFS, Keycloak, and coturn), and `observability`
(Prometheus, Grafana, and Alertmanager). `just compose-up` enables all three profiles. To start a
subset explicitly, use for example `docker compose --profile infrastructure up -d`; the `mahfel`
and `observability` profiles require their dependency profiles as well.

Caddy also has a dedicated `caddy` profile for starting only the reverse proxy:

```bash
just caddy
```

Standalone Caddy starts even when its configured upstreams are not running; use this mode for local
HTTPS certificate work or when proxying to host-run services.

The first media build is slow because `mediasoup-sys` builds the native worker. It also downloads
Python build tooling and Meson wrap dependencies; CI/build networks must permit those downloads or
provide a prewarmed cache/mirror.

Verify endpoints:

```bash
curl https://mahfel.localhost/api/health
curl https://api.mahfel.localhost/v1/status
curl http://localhost:9101/health/ready
curl http://localhost:9102/health/ready
curl http://localhost:9103/health/ready
```

Prometheus is at `http://127.0.0.1:9090`; Grafana is at `http://127.0.0.1:3300`. RabbitMQ management
is available through Caddy at `https://rabbitmq.mahfel.localhost` and directly at
`http://127.0.0.1:15672`; RustFS console is at `http://127.0.0.1:9001`. RabbitMQ authentication uses
`MAHFEL_RABBITMQ_USER` and `MAHFEL_RABBITMQ_PASSWORD`. The Caddy route is local development
convenience only; RabbitMQ management, its AMQP endpoint, and all other admin interfaces must not be
copied into public ingress configuration.

The canonical browser endpoints are `https://mahfel.localhost`, `https://api.mahfel.localhost`, and
the local-only RabbitMQ management route `https://rabbitmq.mahfel.localhost`. Direct loopback ports
`3000`, `8080`, and `15672` remain available for low-level debugging, but normal browser development
must use Caddy HTTPS so secure-context and CORS behavior match deployment assumptions.

`https://caddy.mahfel.localhost` serves a small static page directly from Caddy and is useful for
confirming local HTTPS and certificate trust independently of the application services. Unknown
`*.mahfel.localhost` hosts redirect there, where the available local routes are listed.

The media control gRPC port `50051` is published on loopback for host-run API development only;
it remains a private internal port and must not be exposed through public ingress.

When running web or API directly on the host, set `MAHFEL_CADDY_WEB_UPSTREAM` or
`MAHFEL_CADDY_API_UPSTREAM` to `host.docker.internal:<port>` in `.env`. Caddy then keeps the same
trusted HTTPS domains while proxying to host processes. Ports 80/443 remain owned by Caddy.

RustFS provides the browser-facing S3-compatible endpoint at `https://s3.mahfel.localhost` and its
console at `https://storage.mahfel.localhost`. Containers use the internal
`http://rustfs:9000` endpoint. Keeping separate internal and public endpoint settings prevents
presigned browser URLs from containing an unresolvable Compose service name. The one-shot
`rustfs-init` service creates `MAHFEL_S3_BUCKET` idempotently with the AWS CLI. RustFS data uses the
`rustfs-data` named volume. A previous `minio-data` volume is deliberately not reused because
RustFS's default storage format must not be assumed to be compatible; migrate any valuable local
objects through the S3 API before removing that old volume.

Keycloak provides local OIDC at `https://auth.mahfel.localhost/realms/rastai`. The `mahfel-seed`
service waits for Keycloak, PostgreSQL, and the initialized RustFS bucket, then creates missing development accounts and mirrors
them into PostgreSQL. The accounts are `shinji`, `asuka`, `rei`, `misato`, `ritsuko`, and `admin`;
all use the local-only password `password`. See `deploy/keycloak/README.md` for details. The realm
import defines clients and roles; canonical development accounts live in
`backend/seeds/0002_development_users.toml`.

Browser authentication is provided by `mahfel-web` with Keycloak, and the API verifies the resulting
access token as a resource server (ADR 0019); `GET /v1/auth/me` returns `401` without a valid token.
Development fixtures live separately from schema migrations in `backend/seeds/`. The seed command
parses and validates every selected TOML file before applying it in file-name order. The bundled
fixtures include the local development principal, Keycloak-backed users, and a classroom with a
scheduled session. For host development,
run `just backend seed`, or `just backend seed path/to/seed.toml` to apply one file or a directory.
Compose mounts `backend/seeds/` read-only into `mahfel-seed` and runs
`mahfel-api seed /app/seeds` after migrations. After editing a seed, rerun it against the running
stack without rebuilding an image:

```bash
docker compose --profile infrastructure --profile mahfel run --rm --no-deps mahfel-seed
```

The Minikube seed Job reads the same directory packaged in the API image; rebuild that image and
run `just minikube up` after changing a fixture. A user fixture requires
Keycloak; principal and room fixtures only require PostgreSQL. Re-running fixtures preserves
existing users, passwords, rooms, memberships, and session state. The classroom fixture has stable
IDs in `backend/seeds/0003_development_rooms.toml`; its manager and members refer to the bundled
users. For manual or E2E scenarios, add a TOML file with `kind = "rooms"`, `[[rooms]]` entries
(`id`, `title`, `manager_id`, `visibility`, optional `members`), and optional `[[rooms.sessions]]`
entries (`id`, `title`). Rooms need their referenced actors in PostgreSQL first. Seeded sessions
are scheduled; use the API to exercise lifecycle operations. To add sessions to an existing room,
use `kind = "sessions"` and `[[sessions]]` entries with `id`, `room_id`, and `title`. Never run
development seeds in production.

To seed profile pictures or room/session assets, add a later TOML file with `kind = "files"` and
`[[files]]` entries. Each entry needs a stable `id`, `owner_id`, `context_kind` (`avatar`, `room`, or
`session`), `context_id`, and `path`. An avatar's context ID must equal its owner ID; room and
session owners must be members of the room. Paths may be absolute or relative to the TOML file.
For container deployments, keep relative assets inside `backend/seeds/`; an absolute path must
exist inside the seed container. Compose mounts the checkout directory at `/app/seeds`, while the
Minikube API image copies it to the same path.
PNG and JPEG are accepted for avatars; room and session assets also accept PDF. Files are checked
before database writes, uploaded to S3-compatible storage, and verified through the normal file
completion flow. Re-running a fixture with the same ID and content keeps the ready file. The seed
process needs the configured storage bucket and write access. For example:

```toml
kind = "files"

[[files]]
id = "00000000-0000-4000-8000-000000000301"
owner_id = "00000000-0000-4000-8000-000000000004"
context_kind = "avatar"
context_id = "00000000-0000-4000-8000-000000000004"
path = "assets/teacher.png"

[[files]]
id = "00000000-0000-4000-8000-000000000302"
owner_id = "00000000-0000-4000-8000-000000000004"
context_kind = "session"
context_id = "00000000-0000-4000-8000-000000000201"
path = "/absolute/path/to/slides.pdf"
```

Ready avatars appear in `GET /v1/auth/me` and the active session roster, and the web profile and
session tiles use those signed URLs. Room and session assets are available through the existing
file list and download API for their contexts.

Infrastructure host ports can be changed with the `*_HOST_PORT` values in `.env`. Their defaults
match the port map. For example, use `MAHFEL_REDIS_HOST_PORT=26379` when another local project owns
`6379`; container-to-container URLs continue to use the standard ports.

The Compose file follows the neighboring LMS deployment conventions where useful: explicit
service health dependencies, `unless-stopped`, named infrastructure volumes, an isolated internal
network, a host-routable admin bridge, and loopback binds for private/admin ports. The admin bridge
exists only so Docker can publish those loopback ports; it is not a production ingress model.
Application containers have no persistent volumes.

Stop containers with `just compose-down`. Use `just compose-clean` when intentionally deleting all
local infrastructure data and starting from fresh volumes. Use `just clean` for build, test,
Storybook, and documentation artifacts without touching Docker volumes or environment files.
