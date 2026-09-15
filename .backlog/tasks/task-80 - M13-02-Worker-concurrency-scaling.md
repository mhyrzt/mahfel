---
id: TASK-80
title: M13-02 Worker concurrency scaling
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
labels: []
milestone: m-13
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 88000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Replicas must compete without duplicate effects or shutdown loss.

Implementation scope: Per-queue concurrency/prefetch, leases, idempotency audit, backpressure, drain and autoscaling signals.

Dependencies: M8–M12
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Multiple replicas process each effect once; scale-down safely resolves in-flight jobs.
<!-- AC:END -->
