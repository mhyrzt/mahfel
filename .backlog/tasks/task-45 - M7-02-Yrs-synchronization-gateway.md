---
id: TASK-45
title: M7-02 Yrs synchronization gateway
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
ordinal: 53000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Binary CRDT updates need scalable authenticated transport.

Implementation scope: Yrs doc manager, Yjs sync messages over WebSocket, size/rate limits, cross-replica routing.

Dependencies: M1-05, M7-01
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Two clients on different API replicas converge under concurrent updates.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Design note: Whiteboard and text-editor viewport is shared room state. A participant pan, zoom, or scroll updates the view for everyone; order competing changes deterministically so the latest accepted action wins. Keep shared navigation distinct from per-user awareness.
<!-- SECTION:NOTES:END -->
