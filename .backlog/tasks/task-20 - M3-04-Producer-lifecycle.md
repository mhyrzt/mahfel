---
id: TASK-20
title: M3-04 Producer lifecycle
status: Done
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-20 11:54'
labels: []
milestone: m-3
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 28000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Teachers must publish mic/camera/screen safely; invalid publish must fail clean.

Expanded: Producer registry HashMap<ProducerId,Producer> under transport/room (uuid). Produce(roomId,participantId,transportId,kind,rtpParameters) → validate transport ownership, kind matches mimeType, then transport.produce({kind,rtpParameters,paused false}). Support opus, VP8/H264 with simulcast 3 encodings. Reject RTX. Cleanup on DeleteRoom/CloseParticipant. Metrics producers gauge per-kind (audio/video bounded).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Real browser publishes mic/camera; invalid participant/transport requests fail cleanly.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Extend proto ProduceRequest kind+rtpParameters JSON, ProduceResponse producer_id.
2. Implement backend/crates/rtc/src/producer.rs produce + validation + registry.
3. Wire gRPC Produce map errors.
4. Metrics + GetNodeStats.producers.
5. Test publish Chrome/Firefox via signaling, invalid transport fails INVALID_ARGUMENT.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Extended proto ProduceRequest with kind + rtp_parameters JSON. Added ProducerEntry HashMap in backend/crates/rtc/src/lib.rs with MediaKind Audio/Video mapping, RtpParameters JSON parse, transport.produce(ProducerOptions::new(kind, rtp_params)) with paused false, validation of room/transport ownership, cleanup on delete_room. Wired gRPC Produce in backend/bins/media/src/main.rs with NotFound/InvalidArgument mapping, updated GetNodeStats producers count, added mahfel_media_active_producers gauge on 9102 updated every 5s. Verified: cargo check/clippy -p mahfel-rtc/-p mahfel-media pass, fmt pass, proto fmt applied, rtc-protocol regenerated.
<!-- SECTION:NOTES:END -->
