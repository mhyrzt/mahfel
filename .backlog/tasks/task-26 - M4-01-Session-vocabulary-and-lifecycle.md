---
id: TASK-26
title: M4-01 Session vocabulary and lifecycle
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-20 19:01'
labels:
  - backend
  - domain
  - api
milestone: m-4
dependencies:
  - TASK-3
  - TASK-15
  - TASK-22
documentation:
  - docs/backlog/roadmap.md
priority: high
ordinal: 34000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Rename the live aggregate from ClassroomSession to the generic Session and implement the session lifecycle that anchors M4. A persistent Room can host sessions; a Session has an explicit state machine (scheduled to live to ended, or scheduled to cancelled), at most one live session per room, ownership by a room manager or service principal, and start/end/cancel APIs. Because the image is not deployed beyond local development, migration 0002 is amended in place (classroom_sessions to sessions) and the required local database reset is documented.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Domain, application, PostgreSQL, and API code use Session and sessions, and no ClassroomSession or classroom_sessions identifier remains in backend production code or schema.
- [ ] #2 Invalid transitions are rejected with deliberate errors and leave state unchanged: start from non-scheduled, end from non-live, cancel from live or ended.
- [ ] #3 At most one live session per room holds under concurrent starts, and the losing request receives a conflict rather than a duplicate live session.
- [ ] #4 Start, end, and cancel persist and produce audit events, and ownership is restricted to room managers and service principals deny-by-default.
- [ ] #5 Migration tests cover the renamed schema on a fresh database, and documentation states the amended-0002 checksum consequence and local reset.
<!-- AC:END -->
