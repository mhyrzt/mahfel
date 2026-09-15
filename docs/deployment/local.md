# Local deployment

## Prerequisites

- Docker Engine with Compose v2
- Bun for host frontend work
- Rust 1.95 with rustfmt and Clippy for host backend work
- `just` and MkDocs Material for convenience/documentation
- C++ toolchain, Python/pip, CMake, and pkg-config for native mediasoup builds

Copy `.env.example` to `.env`; values are explicitly local-only. Then run:

```bash
just compose-up
```

The first media build is slow because `mediasoup-sys` builds the native worker. It also downloads
Python build tooling and Meson wrap dependencies; CI/build networks must permit those downloads or
provide a prewarmed cache/mirror.

Verify endpoints:

```bash
curl http://localhost:3000/api/health
curl http://localhost:8080/api/v1/status
curl http://localhost:9101/health/ready
curl http://localhost:9102/health/ready
curl http://localhost:9103/health/ready
```

Prometheus is at `http://127.0.0.1:9090`; Grafana is at `http://127.0.0.1:3300`. RabbitMQ management
is at `http://127.0.0.1:15672`; MinIO console is at `http://127.0.0.1:9001`. All are local/admin
interfaces and must not be copied into public ingress configuration.

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
