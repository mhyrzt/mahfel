---
id: TASK-70
title: M11-03 Mention workflow
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
labels: []
milestone: m-11
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 78000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Users need a bounded explicit interaction before autonomous behavior.

Implementation scope: Chat mention detection, authorization, durable job, response thread/message, progress/cancel and dedupe.

Dependencies: M5-04, M8, M11-01, M11-02
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Mention produces one AI response under duplicate delivery and respects channel permissions.
<!-- AC:END -->
