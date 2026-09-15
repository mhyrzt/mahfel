---
id: TASK-75
title: M12-02 Low-latency STT path
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
labels: []
milestone: m-12
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 83000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Recorded-session STT may be too slow for conversation.

Implementation scope: Streaming/short-form provider capability, endpointing, partial/final text, timeouts and language handling.

Dependencies: M10-01, M12-01
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Final transcript reaches AI within measured target and partials never trigger duplicate answers.
<!-- AC:END -->
