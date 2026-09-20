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

Keycloak provides local OIDC at `https://auth.mahfel.localhost/realms/rastai`. The imported
development accounts are `shinji`, `asuka`, `rei`, `misato`, `ritsuko`, and `admin`; all use the
local-only password `password`. See `deploy/keycloak/README.md` for the fixture identities and the
`mahfel-admin` service account used for Admin API access. The local realm is imported from
`deploy/keycloak/rastai-realm.json` into an in-memory database, so the committed fixture is the
source of truth on every start. Production points the same provider-neutral configuration at its
Keycloak deployment.

Browser authentication is provided by `mahfel-web` with Keycloak, and the API verifies the resulting
access token as a resource server (ADR 0019); `GET /v1/auth/me` returns `401` without a valid token.
Development fixtures that pre-provision local users live separately from schema migrations in
`backend/seeds/`, applied in file-name order. The `mahfel-seed` Compose service applies them (after
migrations) before the API starts; for host development, run `just backend seed`, or
`just backend seed path/to/seed.sql` to apply a single file or directory. Never run seeds in
production.

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
