# Repository structure

| Path | Purpose |
|---|---|
| `backend/bins/api` | Control-plane executable |
| `backend/Dockerfile` | Cargo-chef based API, media, and worker production image targets |
| `backend/bins/media` | Media-plane executable |
| `backend/bins/worker` | Background worker executable |
| `backend/crates/config` | Layered typed configuration |
| `backend/crates/auth` | Provider-neutral OIDC identity normalization and Dex test harness |
| `backend/crates/telemetry` | Logs, metrics, health, shutdown foundation |
| `backend/crates/domain` | Shared durable domain types when genuinely cross-capability |
| `backend/crates/application` | Use cases and outbound port contracts |
| `backend/crates/postgres` | SQLx pool, migrations, and store adapters |
| `backend/migrations` | Forward-only schema migrations |
| `backend/seeds` | Explicit development seed fixtures, applied by `mahfel-api seed` |
| `backend/crates/messaging` | RabbitMQ connection/publishing/consumption boundary |
| `backend/crates/rtc-protocol` | Generated internal gRPC contract |
| `backend/crates/media-client` | API-side gRPC client boundary |
| `backend/crates/rtc` | mediasoup-specific runtime code |
| `frontend/app` | Next.js routes and layouts |
| `frontend/components` | Shared shadcn/ui components |
| `frontend/.storybook` | Storybook framework and preview configuration |
| `frontend/vitest.config.ts` | Vitest/jsdom unit-test configuration |
| `frontend/features` | Feature-oriented product modules as they arrive |
| `frontend/lib` | API, WebSocket, RTC, and collaboration adapters |
| `frontend/Dockerfile` | Next.js production image definition |
| `bruno` | Bruno API collection (placeholder; generated from the OpenAPI document per ADR 0016) |
| `proto` | Source protobuf contracts |
| `deploy` | Container, Prometheus, Grafana, and future platform assets |
| `docs` | MkDocs architecture, decisions, deployment, development, roadmap |

Crates are added when a meaningful boundary has implementation and tests. Chat, files,
collaboration, storage, and AI remain planned module boundaries; authentication now owns the
provider-neutral identity boundary and its local Dex discovery test, while the login/callback flow
remains an M2 implementation task.
