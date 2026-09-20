---
id: TASK-23
title: M3-07 Frontend RTC client
status: In Progress
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-20 12:21'
labels: []
milestone: m-3
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 31000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Device + transport state must survive browser reload/permission deny.

Expanded: Frontend mediasoup-client Device.load(routerRtpCapabilities via API), enumerateDevices, getUserMedia permission UX. Stores for device/sendTransport/recvTransport/producers/consumers/connectionState. Send/recv each via API WS create→connect, handle connectionstatechange with bounded reconnect iceRestart. Error UX: unsupported, NotAllowedError CTA, failed→retry.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Two supported browsers publish/consume and device denial has actionable UI.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add mediasoup-client dep, Device wrapper.
2. Hooks useRtcDevice/useTransports + stores.
3. Permission UI + toasts.
4. E2E two browsers publish/consume.
<!-- SECTION:PLAN:END -->
