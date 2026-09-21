---
id: TASK-125
title: M4-06 Admission and lobby
status: To Do
assignee: []
created_date: '2026-09-20 18:58'
updated_date: '2026-09-20 19:00'
labels:
  - backend
milestone: m-4
dependencies:
  - TASK-27
  - TASK-124
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 133000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Generic admission control replacing the LMS waiting room. A room carries an admission policy of open or lobby. In lobby mode a joining principal enters a bounded waiting state and cannot publish or consume until a principal with admit_participant admits or rejects it. Policy is enforced in join authorization, not only in the UI, and decisions are audited.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Room admission policy supports open and lobby and persists through the API.
- [ ] #2 In lobby mode a joining principal is waiting and cannot publish or consume media until admitted.
- [ ] #3 Admit and reject require the admit_participant capability, and reject terminates the waiting state.
- [ ] #4 Admission is enforced in the join path for both REST and WebSocket, not only in the UI.
- [ ] #5 Admission decisions are audited and the waiting set is bounded and cleared when the session ends.
<!-- AC:END -->
