---
id: TASK-42
title: M6-05 File contexts
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-23 01:13'
labels: []
milestone: m-6
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 50000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Classroom, meeting, chat, avatar, and future recording files need explicit association.

Implementation scope: Context/link tables or typed owner model, lifecycle/deletion rules and list APIs.

Dependencies: M6-03, M6-04
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Each context lists only its files; deleting a link cannot orphan bytes indefinitely.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Design note: Link presentation files to a session or presentation context with stable ordered slide metadata. Recording references must keep the correct artifact resolvable under authorization, retention, and deletion rules.
<!-- SECTION:NOTES:END -->
