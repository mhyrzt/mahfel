# Observability

Each Rust service has a dedicated private operations listener:

- `/metrics` is Prometheus text exposition.
- `/health/live` proves the process and event loop can respond. It does not depend on every remote
  system and should not create restart storms.
- `/health/ready` proves the service can accept useful work and returns 503 otherwise.

API readiness currently requires completed initialization and PostgreSQL. Redis and RabbitMQ are
secondary API dependencies and should later degrade individual capabilities rather than force API
restart loops. Media readiness requires initialized mediasoup workers and a bound control server.
Worker readiness requires RabbitMQ connectivity. Readiness failure removes traffic/work; liveness
failure is the only routine reason for an orchestrator restart.

```mermaid
flowchart LR
  A[mahfel-api :9101] --> P[Prometheus]
  M[mahfel-media :9102] --> P
  W[mahfel-worker :9103] --> P
  Q[RabbitMQ :15692] --> P
  P --> G[Grafana]
  P --> AM[Alertmanager]
  A -. future OTLP spans .-> O[OpenTelemetry collector / Tempo]
  M -. future OTLP spans .-> O
  W -. future OTLP spans .-> O
  Logs[Structured JSON logs + trace IDs] -.-> Operators
  Errors[Future Sentry integration] -.-> Operators
```

Bootstrap exports process metrics and `mahfel_service_ready`, with constant `service` and `version`
labels. The API additionally exports `mahfel_websocket_*` gauges and counters for active and
accepted connections, cap rejections, disconnects by reason, messages, and send drops, and
`mahfel_redis_*` counters for command outcomes, pub/sub messages by direction, and probe failures. Future media metrics cover active rooms, participants, workers, routers, transports,
producers, consumers, ingress/egress bytes and bitrate, packet loss, retransmissions, RTT, and
available outgoing bitrate.

Prometheus labels must stay bounded. Never label metrics with `user_id`, `room_id`, `producer_id`,
email address, or other per-entity identifiers. Put those identifiers in structured logs and traces,
subject to privacy policy, and correlate with trace/request IDs.

## Frontend liveness

`GET /api/health` is the public `mahfel-web` liveness endpoint used by its container healthcheck. It returns HTTP 200 with `status: "ok"`, `service: "mahfel-web"`, the application `version` bundled from `frontend/package.json`, `kind: "liveness"`, a UTC RFC 3339 `timestamp`, integer process `uptimeSeconds`, and `environment` (`NODE_ENV`, or `unknown` when unset). Responses use `Cache-Control: no-store` and execute at request time.

The version identifies the packaged application, not the independently versioned container release tag. Uptime belongs to the responding process and resets on restart; different replicas may have different uptimes. This lightweight probe makes no backend or identity-provider requests and does not establish platform readiness. It exposes no hostnames, credentials, internal URLs, or arbitrary environment variables.
