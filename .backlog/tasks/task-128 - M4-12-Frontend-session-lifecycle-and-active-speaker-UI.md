---
id: TASK-128
title: M4-12 Frontend session lifecycle and active speaker UI
status: In Progress
assignee: []
created_date: '2026-09-20 18:58'
updated_date: '2026-09-25 00:44'
labels:
  - frontend
milestone: m-4
dependencies:
  - TASK-126
  - TASK-26
  - TASK-29
  - TASK-123
  - TASK-132
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 136000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Browser app surfaces for the generic session lifecycle and active speaker: start, end, and cancel controls gated by capability, a live or scheduled or ended state treatment, an active-speaker-driven layout that stays stable under multiple speakers, and a participation summary after the session ends.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A capable principal can start, end, and cancel a session and invalid transitions surface deliberate errors.
- [ ] #2 Scheduled, live, and ended states drive the interface.
- [ ] #3 Active speaker events drive the layout without flicker when multiple speakers overlap.
- [ ] #4 An ended session shows a participation summary sourced from participation records.
- [ ] #5 The engine UI contains no LMS-specific strings, and education copy is optional theming.
- [ ] #6 A capable principal can create an ad-hoc or scheduled session, optionally set or clear the session title and duration, and see the schedule; duration is presented as advisory and never shown as a hard end.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
2026-09-25: Added room/session creation, lifecycle controls, scheduled/live/ended views, active-speaker event handling, and an ended-session participation summary. Editing scheduled title/duration and live multi-client validation remain.
<!-- SECTION:NOTES:END -->
