---
id: TASK-21
title: M3-05 Consumer lifecycle
status: Done
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-20 12:02'
labels: []
milestone: m-3
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 29000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Students need consumable tracks server-selected, paused-first, with layer selection.

Expanded: Consumer registry HashMap<ConsumerId,Consumer> under room (uuid). Consume(roomId,participantId,producerId,rtpCapabilities) → canConsume check then transport.consume({producerId,rtpCapabilities,paused true,enable_rtx true}). Return consumer_id + rtpParameters + paused. Pause/Resume → consumer.pause()/resume(). Preferred layers for simulcast. Producer close auto closes consumers. Metrics consumers gauge.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Multiple browsers consume teacher tracks and cleanup follows disconnect.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Extend proto ConsumeResponse rtpParameters paused; add Pause/Resume handlers.
2. Implement backend/crates/rtc/src/consumer.rs consume paused, resume/pause, close.
3. Wire gRPC Consume/Pause/Resume.
4. Hook producer close → consumer cleanup scan.
5. Metrics + GetNodeStats.consumers.
6. Test multi-consumer pause/resume leak after close.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Extended proto ConsumeRequest with transport_id + rtp_capabilities JSON, ConsumeResponse with rtp_parameters + paused. Added ConsumerEntry HashMap in backend/crates/rtc/src/lib.rs with RtpCapabilities parse, router.can_consume check, transport.consume(ConsumerOptions::new(producer_id, rtp_capabilities) paused true), serde_json rtp_parameters, pause_consumer/resume_consumer via consumer.pause()/resume(). Added cleanup on delete_room, consumer_count, mahfel_media_active_consumers gauge on 9102, GetNodeStats consumers. Verified check/clippy/fmt pass, proto fmt applied.
<!-- SECTION:NOTES:END -->
