---
id: TASK-17
title: M3-01 Worker supervision
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-3
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 25000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Native worker crashes/resource limits need explicit lifecycle behavior.

Implementation scope: Configurable worker count/settings, spawn supervision, failure metrics, readiness, shutdown and restart policy.

Dependencies: M0
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 At least one worker is required for readiness; induced failure is detected and documented.
<!-- AC:END -->
