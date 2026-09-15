# Architecture overview

Mahfel separates browser delivery, authoritative control-plane work, realtime media, and durable
background work. PostgreSQL is the durable source of truth. Redis holds replaceable distributed
coordination state. RabbitMQ transports durable jobs. S3 stores file and media artifacts.

```mermaid
flowchart LR
  Browser[Browser] -->|HTTPS| Web[mahfel-web]
  Browser -->|REST and WebSocket| API[mahfel-api]
  Browser <-->|WebRTC media| Media[mahfel-media]
  Browser -.->|TURN fallback| Turn[coturn]
  Turn -.-> Media
  API --> Postgres[(PostgreSQL)]
  API --> Redis[(Redis)]
  API -->|internal gRPC| Media
  API -->|jobs| Rabbit[(RabbitMQ)]
  API -->|presign| S3[(S3)]
  Browser -->|presigned transfer| S3
  Rabbit --> Worker[mahfel-worker]
  Worker --> Postgres
  Worker --> S3
  Ops[Prometheus] --> API
  Ops --> Media
  Ops --> Worker
  Grafana[Grafana] --> Ops
```

The browser calls `mahfel-api` directly for REST and WebSocket traffic. Next.js is not a general
realtime proxy. Signaling passes through the API, but negotiated media never does: direct WebRTC
flows between browser and media node, falling back through coturn only when network traversal
requires it.

## Data ownership

| System | Role | Authoritative? |
|---|---|---|
| PostgreSQL | Users, rooms, sessions, permissions, chat, file metadata, job outbox | Yes |
| Redis | Presence, leases, node registry, room placement cache, coordination | No |
| RabbitMQ | Durable delivery of commands and jobs | Delivery state only |
| S3 | Files, recordings, thumbnails, exports | Artifact bytes |
| mediasoup nodes | Live media objects and statistics | Only for the active session |

The runtime architecture is modular without becoming a microservice exercise. Domain capabilities
such as auth, chat, files, collaboration, and AI are library/module boundaries inside the three Rust
runtime services, not separately deployed services.
