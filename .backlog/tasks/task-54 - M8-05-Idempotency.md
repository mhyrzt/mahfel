---
id: TASK-54
title: M8-05 Idempotency
status: Done
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-24 00:26'
labels: []
milestone: m-8
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 62000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: AMQP is at-least-once, so duplicate execution is normal.

Implementation scope: Handler idempotency key/store, atomic claim/result, stale lease and per-job guidance.

Dependencies: M8-03, M1-01
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Duplicate delivery cannot duplicate durable effect; crashed claim recovers safely.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Use a unique job-type/idempotency key, atomic PostgreSQL claims and completion, renewed leases, stale-claim recovery, and an idempotent file cleanup handler.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
PostgreSQL test verified unique key, duplicate terminal claim, stale lease recovery, and replay generation; worker end-to-end test passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added atomic job claims and completion, lease recovery, and repeatable file cleanup effects.
<!-- SECTION:FINAL_SUMMARY:END -->
