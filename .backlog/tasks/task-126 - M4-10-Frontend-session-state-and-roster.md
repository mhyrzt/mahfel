---
id: TASK-126
title: M4-10 Frontend session state and roster
status: To Do
assignee: []
created_date: '2026-09-20 18:58'
updated_date: '2026-09-20 19:00'
labels:
  - frontend
milestone: m-4
dependencies:
  - TASK-121
  - TASK-27
  - TASK-125
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 134000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Client-side generic session state for the browser app: connect the WebSocket with a verified token, join and leave a session, render a roster from stable participant identities, and reconcile state across reconnects. Waiting or lobby state from admission policy is represented and hides media controls until admitted. Capability payload gates controls the caller cannot use. Education-facing copy is theming, not core.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The roster reflects join, leave, remove, and block events and survives reconnect without duplicate participants.
- [ ] #2 Reconnect re-establishes signaling and reconciles the stable participant identity.
- [ ] #3 Lobby policy shows a waiting state and no media controls before admission.
- [ ] #4 Controls the caller cannot use are hidden or disabled from the capability payload.
- [ ] #5 Expired or unauthorized tokens produce actionable UX instead of silent failure.
<!-- AC:END -->
