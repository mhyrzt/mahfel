# Repository structure

| Path | Purpose |
|---|---|
| `backend/bins/api` | Control-plane executable |
| `backend/bins/media` | Media-plane executable |
| `backend/bins/worker` | Background worker executable |
| `backend/crates/config` | Layered typed configuration |
| `backend/crates/telemetry` | Logs, metrics, health, shutdown foundation |
| `backend/crates/domain` | Shared durable domain types when genuinely cross-capability |
| `backend/crates/database` | SQLx pool and migrations boundary |
| `backend/crates/messaging` | RabbitMQ connection/publishing/consumption boundary |
| `backend/crates/rtc-protocol` | Generated internal gRPC contract |
| `backend/crates/media-client` | API-side gRPC client boundary |
| `backend/crates/rtc` | mediasoup-specific runtime code |
| `frontend/app` | Next.js routes and layouts |
| `frontend/components` | Shared shadcn/ui components |
| `frontend/features` | Feature-oriented product modules as they arrive |
| `frontend/lib` | API, WebSocket, RTC, and collaboration adapters |
| `proto` | Source protobuf contracts |
| `deploy` | Container, Prometheus, Grafana, and future platform assets |
| `docs` | MkDocs architecture, decisions, deployment, development, roadmap |

Crates are added when a meaningful boundary has implementation and tests. Auth, chat, files,
collaboration, storage, and AI are planned module boundaries but are not empty bootstrap crates.
