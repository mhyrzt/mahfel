---
id: TASK-24
title: M3-08 Screenshare/reconnect
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
ordinal: 32000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Presentation + transient network loss.

Expanded: getDisplayMedia → produce video via sendTransport separate producer. track.onended → close producer. Handle failed→iceRestart, disconnected 5s grace then full rejoin (leave+join) 3 retries else Rejoin button exponential backoff. Rejoin: close transports, re-create device, re-produce.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Screen share starts/stops; brief disconnect recovers or produces explicit rejoin outcome.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Extend producer kind for screen.
2. Add connectionstate/iceRestart logic frontend.
3. Rejoin state machine + UI.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented screenshare/reconnect: useRtc requestDisplay with track.onended cleanup, transport restartIce on failed, reconnect controller disconnected 5s grace then rejoin, failed→onRejoin, reset handling, UI shows rejoin outcome. Verified with transport connectionstatechange.
<!-- SECTION:NOTES:END -->
