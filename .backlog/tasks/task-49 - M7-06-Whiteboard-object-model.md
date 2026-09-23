---
id: TASK-49
title: M7-06 Whiteboard object model
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
ordinal: 57000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Canvas collaboration needs compact mergeable semantic objects.

Implementation scope: Object schema/versioning, create/update/delete ordering, transform batching, viewport and basic shapes.

Dependencies: M7-02–M7-04
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Two browsers edit objects concurrently and converge; reload preserves board.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Design note: Whiteboard pan and zoom are shared room state: a participant changing the viewport moves it for everyone, with competing actions resolved by latest accepted event. Keep viewport events distinct from semantic drawing edits. A presenter-controlled slideshow can reference stored PDFs or ordered images and record slide changes.
<!-- SECTION:NOTES:END -->
