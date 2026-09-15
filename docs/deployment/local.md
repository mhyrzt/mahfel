# Local deployment

## Prerequisites

- Docker Engine with Compose v2
- Bun for host frontend work
- Rust 1.95 with rustfmt and Clippy for host backend work
- `just`, `cargo-nextest`, and MkDocs Material for convenience, tests, and documentation
- C++ toolchain, Python/pip, CMake, and pkg-config for native mediasoup builds

Copy `.env.example` to `.env`; values are explicitly local-only. Then run:

```bash
just compose-up
```

`just compose-up` waits for Caddy's local CA to exist and runs `just trust-local-ca`. The trust
step uses `sudo` to install the generated root certificate into Fedora/RHEL, Debian/Ubuntu, or
macOS system trust. This is mandatory: do not bypass browser certificate warnings. Restart open
browsers after the first trust operation. Caddy keeps its CA in the persistent `caddy-data` volume,
so subsequent runs retain the same trusted root unless that volume is intentionally deleted.

The first media build is slow because `mediasoup-sys` builds the native worker. It also downloads
Python build tooling and Meson wrap dependencies; CI/build networks must permit those downloads or
provide a prewarmed cache/mirror.

Verify endpoints:

```bash
curl https://mahfel.localhost/api/health
curl https://api.mahfel.localhost/api/v1/status
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

RustFS provides the browser-facing S3-compatible endpoint at `https://s3.mahfel.localhost` and its
console at `https://storage.mahfel.localhost`. Containers use the internal
`http://rustfs:9000` endpoint. Keeping separate internal and public endpoint settings prevents
presigned browser URLs from containing an unresolvable Compose service name. The one-shot
`rustfs-init` service creates `MAHFEL_S3_BUCKET` idempotently with the AWS CLI. RustFS data uses the
`rustfs-data` named volume. A previous `minio-data` volume is deliberately not reused because
RustFS's default storage format must not be assumed to be compatible; migrate any valuable local
objects through the S3 API before removing that old volume.

Infrastructure host ports can be changed with the `*_HOST_PORT` values in `.env`. Their defaults
match the port map. For example, use `MAHFEL_REDIS_HOST_PORT=26379` when another local project owns
`6379`; container-to-container URLs continue to use the standard ports.

The Compose file follows the neighboring LMS deployment conventions where useful: explicit
service health dependencies, `unless-stopped`, named infrastructure volumes, an isolated internal
network, a host-routable admin bridge, and loopback binds for private/admin ports. The admin bridge
exists only so Docker can publish those loopback ports; it is not a production ingress model.
Application containers have no persistent volumes.

Stop containers with `just compose-down`. Add `--volumes` manually only when intentionally deleting
local infrastructure data.
