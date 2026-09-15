# Networking and security boundaries

## Port map

| Component | Container port(s) | Local host mapping | Exposure |
|---|---:|---:|---|
| `mahfel-web` | 3000/TCP | 3000 | Public web |
| `mahfel-api` | 8080/TCP | 8080 | Public REST/WebSocket |
| `mahfel-api` operations | 9101/TCP | 127.0.0.1:9101 | Private |
| `mahfel-media` gRPC | 50051/TCP | none | Private internal |
| `mahfel-media` RTC | 40000–40003/UDP and TCP | same ports | Public media |
| `mahfel-media` operations | 9102/TCP | 127.0.0.1:9102 | Private |
| `mahfel-worker` operations | 9103/TCP | 127.0.0.1:9103 | Private |
| PostgreSQL | 5432/TCP | 127.0.0.1:5432 | Private |
| Redis | 6379/TCP | 127.0.0.1:6379 | Private |
| RabbitMQ AMQP | 5672/TCP | 127.0.0.1:5672 | Private |
| RabbitMQ management | 15672/TCP | 127.0.0.1:15672 | Local/admin only |
| RabbitMQ metrics | 15692/TCP | 127.0.0.1:15692 | Private |
| MinIO API/console | 9000/9001 TCP | loopback same | Local/admin only |
| coturn | 3478 UDP/TCP | same | Public traversal |
| coturn relay | 49160–49200/UDP | same | Public local relay range |
| Prometheus | 9090/TCP | 127.0.0.1:9090 | Private |
| Grafana | 3000/TCP | 127.0.0.1:3300 | Private/admin |
| Alertmanager | 9093/TCP | 127.0.0.1:9093 | Private |

The listed local mappings are defaults. Infrastructure host-side ports can be overridden with the
documented `MAHFEL_*_HOST_PORT` variables when another development stack owns a default; service
ports and service-to-service URLs do not change.

Ports 50051 and 9101–9103, databases, caches, AMQP, management, and observability endpoints are not
public Internet endpoints. A dedicated operations port is an isolation boundary only when the
platform firewall/network policy keeps it private.

One mediasoup `WebRtcServer` is created per worker. Bootstrap assigns 40000–40003 and enables TCP
fallback on each same-numbered RTC port. UDP is preferred. The worker count and range are
configurable; the range must contain at least one port per worker.

Production coturn additionally needs TLS certificate handling for 5349/TCP and may expose TURN/TLS
on 443 where platform policy allows it. The local profile intentionally disables TLS/DTLS and uses
49160–49200 only to keep development port mappings tractable.
