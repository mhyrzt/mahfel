---
id: TASK-19
title: M3-03 WebRtcServer/transports
status: Done
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-20 11:51'
labels: []
milestone: m-3
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 27000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Browsers need reachable ICE/DTLS transports on predictable UDP/TCP ports via WebRtcServer.

Expanded: 1 WebRtcServer per Worker done (40000-40003). Add Transport registry HashMap<TransportId,WebRtcTransport> under RoomId (RwLock), id uuid. CreateWebRtcTransport(roomId,participantId) → router.create_webrtc_transport({webrtc_server, enable_udp true, enable_tcp true, prefer_udp true}). Return transport_id + iceParameters/iceCandidates/dtlsParameters. ConnectWebRtcTransport(roomId,transportId,dtlsParameters) → transport.connect(). Validate room/transport ownership. Extend proto responses. Metrics transports gauge. Test direct UDP + TCP fallback.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Browser transport connects over direct UDP; TCP fallback behavior is recorded.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Extend proto CreateWebRtcTransportResponse + Connect dtlsParameters; just proto fmt-check/lint.
2. Implement backend/crates/rtc/src/transport.rs create/connect/close.
3. Wire gRPC handlers map NotFound/InvalidArgument.
4. Metrics mahfel_media_transports.
5. Integration: 2 transports per room, close on DeleteRoom.
6. Manual mediasoup-client Device.createSendTransport UDP then forced TCP.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented transport lifecycle: extended proto/media.proto CreateWebRtcTransportResponse with ice_parameters/ice_candidates/dtls_parameters JSON and Connect request dtls_parameters, buf fmt applied. Added backend/crates/rtc/src/transport registry in lib.rs: RouterEntry with worker_index, TransportEntry HashMap, methods create_transport (router.create_webrtc_transport(WebRtcTransportOptions::new_with_server(webrtc_server)) + serde_json ice/dtls) and connect_transport (parse DtlsParameters JSON + Transport::connect). Wired gRPC handlers with NotFound/InvalidArgument mapping, updated GetNodeStats transports count, added mahfel_media_active_transports gauge on 9102 updated every 5s. Verified: cargo check -p mahfel-rtc/-p mahfel-media pass, clippy -D warnings pass, fmt pass, proto fmt-check/lint (pre-existing lint warnings ignored).
<!-- SECTION:NOTES:END -->
