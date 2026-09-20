---
id: TASK-123
title: M4-03 Participation records
status: To Do
assignee: []
created_date: '2026-09-20 18:58'
labels:
  - backend
milestone: m-4
dependencies:
  - TASK-27
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 131000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Replace the LMS attendance concept with generic, durable participation records. A participant (M4-02) accumulates join/leave intervals per session; intervals aggregate to connected duration, are queryable per session and per participant, and can be corrected with an audited history. Participation is durable PostgreSQL state and is never derived from ephemeral Redis presence.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Every connect and disconnect opens or closes a participation interval, and intervals for one participant never overlap.
- [ ] #2 Intervals aggregate to a per-participant total connected duration and are queryable by session and by participant.
- [ ] #3 The reconnect grace policy is explicit and documented, and reconnects within the window do not fragment records.
- [ ] #4 Corrections are audited and preserve the original interval history.
- [ ] #5 No interval remains open after the session ends, including after abrupt disconnects or API restarts.
<!-- AC:END -->
