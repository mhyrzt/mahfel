---
id: TASK-52
title: M8-03 Consumer acknowledgements
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-8
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 60000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Workers must distinguish success, transient failure, and poison input.

Implementation scope: Manual ack, typed handler result, cancellation, prefetch/concurrency and graceful drain.

Dependencies: M8-01
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Successful job acked; shutdown stops intake and finishes/returns in-flight jobs safely.
<!-- AC:END -->
