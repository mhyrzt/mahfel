---
id: TASK-23
title: M3-07 Frontend RTC client
status: In Progress
assignee:
  - '@codex'
created_date: '2026-09-08 00:31'
updated_date: '2026-09-27 03:29'
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

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented frontend RTC client: added mediasoup-client 3.23.2, lib/rtc/device.ts (createDevice via Device.load, isDeviceSupported via detectDevice), transport.ts (createSendTransport/createRecvTransport with onConnect/onProduce, connectionstatechange failed→restartIce), useRtc.ts (permission handling for getUserMedia/getDisplayMedia with NotAllowedError actionable UI, deviceReady state), reconnect.ts (disconnected 5s grace 3 retries, failed→rejoin), websocket/client.ts RtcWsClient with id correlation, exports. Verified bun install, tsc --noEmit pass.

2026-09-26 live session audit: the session toolbar has no RTC callbacks, and the frontend RTC helpers are not connected to the session UI. The media transport response does not provide router RTP capabilities, and producer discovery/fanout is missing for remote consumption. The prior two-browser acceptance claim has no live proof. Reopen until microphone, camera, screen share, remote playback, permission denial, and reconnect are verified with two browsers.

2026-09-27: Connected live microphone, camera, and screen publication to the toolbar, added router RTP capabilities, a media-node producer snapshot RPC, WebSocket list_producers signaling, browser receive transport and paused-first consumer resume, remote audio/video display, and focused receive-hook tests. Frontend checks and Rust check/clippy pass. Two-browser live proof and reconnect/ICE behavior remain unverified because the running Compose images were not replaced during an active local session.
<!-- SECTION:NOTES:END -->
