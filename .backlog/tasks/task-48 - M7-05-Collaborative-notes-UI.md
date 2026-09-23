---
id: TASK-48
title: M7-05 Collaborative notes UI
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-23 01:13'
labels: []
milestone: m-7
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 56000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: The first document type validates text collaboration end to end.

Implementation scope: Yjs frontend provider, note editor choice/adapter, reconnect/resync, permission UX.

Dependencies: M7-02–M7-04
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Multiple users edit simultaneously, reconnect, converge, and survive backend restart.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Design note: The collaborative text editor shares scroll or viewport position room-wide. When one participant navigates, all follow the same view; the ordered shared-position changes are part of the replay timeline when recording is enabled.
<!-- SECTION:NOTES:END -->
