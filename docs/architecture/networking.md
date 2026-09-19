# Networking and security boundaries

## Port map

| Component | Container port(s) | Local host mapping | Exposure |
|---|---:|---:|---|
| Caddy local edge | 80/443 TCP | 127.0.0.1:80/443 | Trusted local HTTPS |
| `mahfel-web` | 3000/TCP | 127.0.0.1:3000 | Local debug; normal access through Caddy |
| `mahfel-api` | 8080/TCP | 127.0.0.1:8080 | Local debug; normal access through Caddy |
| `mahfel-api` operations | 9101/TCP | 127.0.0.1:9101 | Private |
| `mahfel-media` gRPC | 50051/TCP | none | Private internal |
| `mahfel-media` RTC | 40000–40003/UDP and TCP | same ports | Public media |
| `mahfel-media` operations | 9102/TCP | 127.0.0.1:9102 | Private |
| `mahfel-worker` operations | 9103/TCP | 127.0.0.1:9103 | Private |
| PostgreSQL | 5432/TCP | 127.0.0.1:5432 | Private |
| Redis | 6379/TCP | 127.0.0.1:6379 | Private |
| RabbitMQ AMQP | 5672/TCP | 127.0.0.1:5672 | Private |
| RabbitMQ management | 15672/TCP | 127.0.0.1:15672; `rabbitmq.mahfel.localhost` through Caddy | Local/admin only |
| RabbitMQ metrics | 15692/TCP | 127.0.0.1:15692 | Private |
| RustFS API/console | 9000/9001 TCP | loopback same; HTTPS through Caddy | Local/admin only |
| Keycloak OIDC | 8080/TCP | 127.0.0.1:8080; HTTPS through Caddy | Local/test only |
| coturn | 3478 UDP/TCP | same | Public traversal |
| coturn relay | 49160–49200/UDP | same | Public local relay range |
| Prometheus | 9090/TCP | 127.0.0.1:9090 | Private |
| Grafana | 3000/TCP | 127.0.0.1:3300 | Private/admin |
| Alertmanager | 9093/TCP | 127.0.0.1:9093 | Private |

The listed local mappings are defaults. Infrastructure host-side ports can be overridden with the
documented `MAHFEL_*_HOST_PORT` variables when another development stack owns a default; service
ports and service-to-service URLs do not change.

Local browser traffic terminates at Caddy using its internal CA. The CA certificate must be copied
from Caddy's persistent data volume and installed with `just trust-local-ca`; ignoring certificate
errors is not a supported development mode.

Local OIDC uses `https://auth.mahfel.localhost/realms/rastai`, which Caddy proxies to the
Compose-only Keycloak service. Keycloak is also the production provider; only the issuer URL and
client secret differ between the two through configuration.

Browser REST traffic is authenticated in `mahfel-web` and forwarded to `mahfel-api` by its narrow
BFF (ADR 0019); WebSocket traffic connects to the API directly. The BFF is not a general proxy, so
media, WebSocket, and presigned S3 traffic never pass through Next.js.

Caddy routes `s3.mahfel.localhost` to the RustFS S3 API, `storage.mahfel.localhost` to its console,
and `rabbitmq.mahfel.localhost` to the local RabbitMQ management UI. The latter is an operator
convenience only. The RustFS route is necessary for future browser-to-S3 uploads from the HTTPS web
application. Media UDP/TCP, internal gRPC, operations endpoints, databases, AMQP, and Prometheus,
Grafana, and Alertmanager do not pass through Caddy.

Ports 50051 and 9101–9103, databases, caches, AMQP, management, and observability endpoints are not
public Internet endpoints. A dedicated operations port is an isolation boundary only when the
platform firewall/network policy keeps it private.

One mediasoup `WebRtcServer` is created per worker. Bootstrap assigns 40000–40003 and enables TCP
fallback on each same-numbered RTC port. UDP is preferred. The worker count and range are
configurable; the range must contain at least one port per worker.

Production coturn additionally needs TLS certificate handling for 5349/TCP and may expose TURN/TLS
on 443 where platform policy allows it. The local profile intentionally disables TLS/DTLS and uses
49160–49200 only to keep development port mappings tractable.
