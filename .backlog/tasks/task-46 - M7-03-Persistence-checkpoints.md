---
id: TASK-46
title: M7-03 Persistence/checkpoints
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-7
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 54000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: In-memory CRDT state must survive restarts without unbounded replay.

Implementation scope: Update log versus snapshot design, atomic checkpoint, compaction, load/recovery and corruption handling.

Dependencies: M7-02
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 State restores after restart; compaction preserves state vector; recovery test covers interrupted checkpoint.
<!-- AC:END -->
