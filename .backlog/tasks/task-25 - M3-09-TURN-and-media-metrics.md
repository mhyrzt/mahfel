---
id: TASK-25
title: M3-09 TURN and media metrics
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-20 10:41'
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
- [ ] #1 TURN fallback works; required room/participant/transport/bitrate/loss/RTT metrics are scrapeable.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. TURN credential endpoint + coturn check.
2. Stats exporter polling getStats/dump every 15s into Gauges.
3. Verify forced relay + scrape.
<!-- SECTION:PLAN:END -->
