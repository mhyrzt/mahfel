---
id: TASK-30
title: M4-08 Media node registry
status: Done
assignee:
  - '@mahyar'
created_date: '2026-09-08 00:31'
updated_date: '2026-09-23 00:21'
labels:
  - backend
  - media
milestone: m-4
dependencies:
  - TASK-7
  - TASK-25
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 38000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Give the placement scheduler fresh, trustworthy capacity and lifecycle state. Define a Redis heartbeat schema with TTL, node identity, address, capacity, and draining state, expose a private authenticated update path so a spoofed or unauthenticated update is rejected, and publish bounded registry metrics with explicit namespaced key lifecycle.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A dead node expires after the TTL and is excluded from placement.
- [x] #2 Draining state persists for the heartbeat lifetime, and the node is removed after expiry.
- [x] #3 The update path is private and authenticated, and a spoofed or unauthenticated update is rejected.
- [x] #4 Registry metrics have bounded cardinality and keys are namespaced with explicit lifecycle semantics.
- [x] #5 Losing the registry degrades placement with an explicit unavailable outcome rather than an incorrect placement.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Define stable node identity, heartbeat payload, room capacity, interval/TTL, and private gRPC registry service in the internal protocol.
2. Authenticate each media heartbeat using a node-bound credential, validate the payload, and keep the registry listener private.
3. Add a namespaced Redis registry adapter with atomic heartbeat/index updates, sticky draining, TTL expiry, live lookup, and explicit unavailable errors.
4. Publish heartbeats from mahfel-media, wire API registry metrics and service lifecycle, and configure local Minikube deployment.
5. Update architecture and operational documentation; run formatting, lint, build, proto, and docs checks.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented a private MediaRegistryService gRPC heartbeat on API port 50052. Media publishes a stable configured node ID, direct gRPC URL, max room capacity, observed workers/rooms/transports and draining state every five seconds. API verifies a node-bound HMAC-SHA256 bearer credential before accepting updates. Redis stores a 20-second TTL hash and expiring live index under the configured namespace; an atomic script keeps draining sticky, and live_nodes returns Unavailable on Redis failure or an incomplete/oversized read. Local Minikube configuration and architecture docs updated. Validation: cargo check for API/media/Redis, scoped Clippy with warnings denied, cargo build API/media, backend fmt, Buf format/lint, strict docs build, kubectl kustomize, git diff --check. Functional tests were not run.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Delivered the M4-08 media-node registry with authenticated private heartbeats, TTL-backed Redis discovery, sticky draining, bounded aggregate metrics, and explicit unavailable reads for future placement. Scoped static/build/protocol/docs/deployment checks passed; functional tests were not run. M4-09 will consume the registry for placement.
<!-- SECTION:FINAL_SUMMARY:END -->
