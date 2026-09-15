---
id: TASK-78
title: M12-05 Turn interruption and mute
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
labels: []
milestone: m-12
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 86000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Classroom conversation requires stopping speech and user control.

Implementation scope: End-to-end cancel propagation, barge-in policy, queueing, teacher/user mute and producer pause/close.

Dependencies: M12-01–M12-04
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 User interrupts in-flight response; muted AI emits no audible track; resources clean up.
<!-- AC:END -->
