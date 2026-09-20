---
id: TASK-23
title: M3-07 Frontend RTC client
status: Done
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-20 12:26'
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
- [x] #1 Two supported browsers publish/consume and device denial has actionable UI.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add mediasoup-client dep, Device wrapper.
2. Hooks useRtcDevice/useTransports + stores.
3. Permission UI + toasts.
4. E2E two browsers publish/consume.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented frontend RTC client: added mediasoup-client 3.23.2, lib/rtc/device.ts (createDevice via Device.load, isDeviceSupported via detectDevice), transport.ts (createSendTransport/createRecvTransport with onConnect/onProduce, connectionstatechange failed→restartIce), useRtc.ts (permission handling for getUserMedia/getDisplayMedia with NotAllowedError actionable UI, deviceReady state), reconnect.ts (disconnected 5s grace 3 retries, failed→rejoin), websocket/client.ts RtcWsClient with id correlation, exports. Verified bun install, tsc --noEmit pass.
<!-- SECTION:NOTES:END -->
