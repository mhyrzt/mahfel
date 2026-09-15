---
id: TASK-47
title: M7-04 Awareness/presence
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-7
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 55000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Cursors/selections are useful but ephemeral and potentially high-volume.

Implementation scope: Yjs awareness relay, TTL cleanup, privacy fields, throttling and no durable persistence.

Dependencies: M7-02, M1-06
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Disconnect/TTL removes awareness and bursts stay within bounded queues.
<!-- AC:END -->
