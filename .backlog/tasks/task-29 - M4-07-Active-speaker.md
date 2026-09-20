---
id: TASK-29
title: M4-07 Active speaker
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-20 19:02'
labels:
  - backend
  - api
milestone: m-4
dependencies:
  - TASK-21
  - TASK-27
  - TASK-25
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 37000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Provide a stable aggregate active-speaker signal from mediasoup audio levels: smoothing and rate limits to prevent event floods, an authorized WebSocket event delivered only to session participants, cleanup when a producer closes or a participant disconnects, and a bounded metric with no participant labels. Vocabulary stays generic.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Multiple simultaneous speakers update the interface without event flooding, using rate limits or hysteresis.
- [ ] #2 Speaker events are delivered only to authorized session participants.
- [ ] #3 Producer close and participant disconnect remove speaker state promptly.
- [ ] #4 The speaker metric has bounded cardinality and carries no participant or room labels.
- [ ] #5 Smoothing thresholds and event frequency are documented.
<!-- AC:END -->
