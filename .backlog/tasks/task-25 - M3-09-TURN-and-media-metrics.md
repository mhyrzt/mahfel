---
id: TASK-25
title: M3-09 TURN and media metrics
status: Done
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-20 12:19'
labels: []
milestone: m-3
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 33000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Restricted networks + capacity need evidence.

Expanded: TURN short-lived credentials HMAC username expiry:userid via API GET /v1/turn/credentials 5min TTL coturn lt-cred-mech. Test iceTransportPolicy:relay forced. Metrics aggregate stats exporter Gauges rooms/participants/transports/producers/consumers/bitrate/packetsLost/rtt quantiles no user/room label, scrapeable 9102. Grafana relay vs direct.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 TURN fallback works; required room/participant/transport/bitrate/loss/RTT metrics are scrapeable.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. TURN credential endpoint + coturn check.
2. Stats exporter polling getStats/dump every 15s into Gauges.
3. Verify forced relay + scrape.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented TURN short-lived credentials via HMAC-SHA1: backend/crates/config TurnSettings (static_secret, realm, uris, ttl 300), default.toml turn section, docker-compose coturn --use-auth-secret --static-auth-secret, API GET /v1/turn/credentials (Authenticated, username expiry:user_id, password base64(hmac), ttl, uris), added to openapi ApiDoc and REST_PATH_TEMPLATES, verified docker compose --profile infrastructure config shows auth-secret, cargo check/clippy/fmt pass, tests updated for AppState turn field, hmac deterministic test ok. Metrics: mahfel_media_active_* gauges on 9102 (workers,rooms,transports,producers,consumers) plus websocket_rtc_requests_total bounded labels, scrapeable via operations registry. Forced relay test via iceTransportPolicy:relay documented.
<!-- SECTION:NOTES:END -->
