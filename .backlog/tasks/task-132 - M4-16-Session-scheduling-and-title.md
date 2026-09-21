---
id: TASK-132
title: M4-16 Session scheduling and title
status: To Do
assignee: []
created_date: '2026-09-20 19:44'
labels:
  - backend
  - api
milestone: m-4
dependencies:
  - TASK-26
  - TASK-124
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 140000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Give a session an optional single-occurrence schedule and an optional custom title. A session can be ad-hoc (no schedule) or scheduled with an optional start instant and an optional duration; both are optional and the duration is advisory only, never auto-ending a session. The session title is optional and falls back to the room title when unset; the room title stays required. These fields extend the sessions table created by the amended migration 0002 rather than adding a separate schedule table. Setting the schedule or title is a management action gated by capability and audited.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A session may be ad-hoc or scheduled with an optional scheduled start instant and an optional positive bounded duration, both validated in the domain and by the database.
- [ ] #2 Session title is optional and validated like a room title, and read responses fall back to the room title when the session title is unset.
- [ ] #3 The schedule never constrains start: a capable principal can start an ad-hoc or scheduled session at any time.
- [ ] #4 Duration is advisory and never auto-ends a live session; only an explicit end transitions it.
- [ ] #5 Setting or changing the schedule and title requires the manage capability for the target and is persisted and audited.
<!-- AC:END -->
