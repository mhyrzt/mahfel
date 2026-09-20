---
id: TASK-28
title: M4-05 Moderation controls
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-20 19:02'
labels:
  - backend
  - api
milestone: m-4
dependencies:
  - TASK-124
  - TASK-27
  - TASK-22
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 36000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Capability-gated moderation that works for any vertical. Implement mute request versus enforced mute, remove and block, publish and screen permission gating, propagation of moderation state over WebSocket, and audit events. An education deployment maps its teacher role onto these capabilities rather than adding new engine roles.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A participant without the required capability cannot mute or remove others, and a bypass attempt is rejected server-side.
- [ ] #2 Mute request and enforced mute are distinct, and enforcement pauses or closes the relevant producer through media control.
- [ ] #3 Remove and block are distinct, and a blocked principal cannot rejoin while the block is active.
- [ ] #4 Publish and screen gating applies to both new and existing producers.
- [ ] #5 Every moderation action emits an audit event and propagates to affected clients over WebSocket, verified by an integration test.
<!-- AC:END -->
