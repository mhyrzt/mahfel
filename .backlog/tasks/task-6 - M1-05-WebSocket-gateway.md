---
id: TASK-6
title: M1-05 WebSocket gateway
status: Done
assignee:
  - '@mahyar'
created_date: '2026-09-08 00:31'
updated_date: '2026-09-19 15:54'
labels: []
milestone: m-1
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 14000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Signaling and realtime capabilities need a scalable connection boundary.

Implementation scope: Upgrade route, connection lifecycle, heartbeat, typed envelope/versioning, bounded queues, graceful close, metrics.

Dependencies: M1-03
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Multiple clients connect, exchange a test event, detect disconnect, and expose aggregate metrics.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Deps: enable axum ws; add tokio-tungstenite + futures-util as dev-deps.
2. Transport modules under http/ws: envelope (versioned JSON), config (WsConfig), metrics (WsMetrics).
3. Auth seam: UpgradeAuthenticator (auth-first-message), deny by default.
4. Connection lifecycle: bounded outbound queue, write timeout, heartbeat, graceful close.
5. Registry: in-process fan-out + global connection cap.
6. Route GET /v1/ws and composition wiring (shutdown watch before AppState, register metrics).
7. Tests: envelope unit + real-socket integration (connect, echo, broadcast, 4401, disconnect metrics, heartbeat, shutdown).
8. Validate and record.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented the gateway as transport code under bins/api/src/http/ws (config, envelope, auth, registry, connection, metrics, state). Protocol: JSON envelope with v=1, internally tagged type, optional id; client frames auth/echo/broadcast, server frames welcome/echo/broadcast/error/bye. Auth seam UpgradeAuthenticator with deny-by-default DevAuthenticator (M2 ticket validation drops in without touching the gateway); credentials accepted from a bearer header or an auth-first-message within auth_timeout. Lifecycle: bounded per-connection outbound queue (try_send on fan-out, no head-of-line blocking), write timeout closes a wedged client with 1013, server ping every heartbeat_interval and close 4408 after heartbeat_timeout, graceful close 1001 on the shared shutdown watch. Process-local fan-out registry with an atomic reservation cap (503 before upgrade when full). Metrics registered on the operations registry: mahfel_websocket_connections_active/total/rejected_total, disconnects_total{reason}, messages_received/sent_total, send_drops_total. Limits come from [api.ws] in default.toml (max_connections 10000, outbound_capacity 64, write 10s, heartbeat 15s/45s, auth 10s, 64 KiB frames). Route GET /v1/ws.

Validation: cargo clippy --workspace --all-targets -D warnings clean; mahfel-api tests 45 passed (12 new: 5 envelope unit tests, 7 real-socket integration tests covering echo, broadcast fan-out, unauthenticated close, disconnect capacity release, heartbeat timeout, graceful shutdown, and cap rejection); workspace lib/bins tests pass. Boot smoke on a disposable PostgreSQL container: migrate + seed + serve reached 'API service ready', SIGTERM exited 0, and /metrics exposed the mahfel_websocket_* families.

Known boundaries: fan-out is process-local (cross-replica delivery and presence are M1-06 Redis work); the interim DevAuthenticator ignores the token and only grants the configured development principal, so it denies by default when that principal is unset; heartbeat Pings cannot be observed by a client that never reads, which is exactly the idle case the timeout is meant to close.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Shipped the WebSocket gateway at GET /v1/ws: a versioned JSON envelope, a deny-by-default UpgradeAuthenticator seam (bearer header or auth-first-message), heartbeat ping/timeout, a bounded per-connection outbound queue with a write timeout, a process-local fan-out registry with an atomic connection cap, and mahfel_websocket_* metrics on the operations registry. Limits come from the api.ws config section. Verified with workspace clippy, real-socket integration tests (echo, broadcast, auth denial, capacity release, heartbeat timeout, graceful shutdown, cap rejection), and a boot smoke; committed in d08b1d0 with docs and backlog.
<!-- SECTION:FINAL_SUMMARY:END -->
