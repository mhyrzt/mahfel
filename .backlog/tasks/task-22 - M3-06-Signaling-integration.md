---
id: TASK-22
title: M3-06 Signaling integration
status: In Progress
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-20 12:02'
labels: []
milestone: m-3
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 30000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Browser must never call private gRPC 50051 directly; all signaling via API WS.

Expanded: API WS envelope v1 backend/bins/api/src/http/ws — add typed RTC messages createTransport/connectTransport/produce/consume/pause/resume/closeParticipant with requestId correlation. Auth via TokenVerifier → principal then RBAC via RoleBasedAuthorizer (room member only). Bridge via GrpcMediaGateway lazy channel 5s deadline + 2 retries UNAVAILABLE only, map Status to WS error envelope. Media bytes bypass API (SRTP browser→4000x). Metrics ws_rtc_messages_total bounded label + gateway latency.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 End-to-end signaling works; gRPC remains private; media bytes bypass API.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Define signaling schema backend/crates/rtc-protocol/src/signaling.rs.
2. Extend media-client GrpcMediaGateway with transport/producer/consumer + timeouts.
3. Add API WS handler backend/bins/api/src/http/ws/rtc.rs authorize+correlation+bridge.
4. WS metrics bounded.
5. Integration two WS clients different replicas.
6. Security test gRPC not via Caddy, direct grpcurl outside fails.
<!-- SECTION:PLAN:END -->
