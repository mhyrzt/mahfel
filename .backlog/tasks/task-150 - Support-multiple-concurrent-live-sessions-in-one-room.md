---
id: TASK-150
title: Support multiple concurrent live sessions in one room
status: To Do
assignee: []
created_date: '2026-09-27 02:53'
labels:
  - backend
  - frontend
  - rtc
dependencies: []
references:
  - docs/architecture/components.md
  - docs/architecture/scaling.md
  - docs/architecture/chat.md
ordinal: 158000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Allow a durable room to host two or more live sessions at the same time. This supersedes the one-live-session-per-room invariant delivered by TASK-26. Keep each session independently joinable and endable, with isolated RTC media and session-scoped events. Define whether one actor may participate in several sessions at once and whether room chat remains shared before implementation.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Two scheduled sessions in the same room can both start and remain live under concurrent requests; invalid lifecycle transitions still return conflicts.
- [ ] #2 Joining, ending, or cleaning up one session does not change participants, media, collaboration state, or speaker events in another live session in the same room.
- [ ] #3 Session-targeted moderation affects only its selected session; room-wide membership and block changes are enforced across every live session in the room.
- [ ] #4 Room members can discover and select among all live sessions, and the start UI no longer treats another live session as a conflict.
- [ ] #5 The multi-session participation rule and shared-room-chat behavior are documented and covered by focused tests.
- [ ] #6 Database migration, API and UI contracts, architecture and roadmap documentation, and relevant backend/frontend tests are updated for the new invariant.
<!-- AC:END -->
