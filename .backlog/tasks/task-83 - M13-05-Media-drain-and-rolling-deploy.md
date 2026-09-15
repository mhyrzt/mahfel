---
id: TASK-83
title: M13-05 Media drain and rolling deploy
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
labels: []
milestone: m-13
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 91000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Live rooms cannot be moved like stateless HTTP requests.

Implementation scope: Draining registry state, scheduler exclusion, deadline, termination grace, optional room warning and rollout runbook.

Dependencies: M4-05, M4-06
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 New rooms avoid drain; existing rooms finish within policy; node exits gracefully.
<!-- AC:END -->
