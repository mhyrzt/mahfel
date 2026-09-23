---
id: TASK-35
title: M5-04 Reactions/mentions/threads
status: Done
assignee:
  - '@codex'
created_date: '2026-09-08 00:31'
updated_date: '2026-09-23 14:58'
labels: []
milestone: m-5
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 43000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Classroom discussion needs structured interaction, not opaque text hacks.

Implementation scope: Reaction uniqueness, mention validation/notification event, parent/reply constraints and thread queries.

Dependencies: M5-02, M5-03
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Concurrent reactions converge; invalid mentions/parents rejected; thread history paginates.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add one-reaction-per-actor persistence and structured mention validation for current room members. 2. Add root/reply constraints and stable thread pagination. 3. Expose REST contracts and committed reaction/reply events; postpone mention notifications as agreed. 4. Test concurrent reactions, invalid parents/mentions, and thread pagination with PostgreSQL.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
One reaction choice per actor/message, current-member mentions, one-level replies, and stable thread paging; mention notifications deferred by decision. PostgreSQL concurrent reaction, invalid mention/parent, and pagination tests passed in full integration suite.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented reactions, mentions, replies, and REST/event contracts with PostgreSQL edge-case coverage.
<!-- SECTION:FINAL_SUMMARY:END -->
