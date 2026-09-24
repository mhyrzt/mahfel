---
id: TASK-53
title: M8-04 Retry and DLQ
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
ordinal: 61000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Infinite redelivery hides poison jobs and creates load.

Implementation scope: Bounded delayed retries, attempt metadata, exponential policy, DLX/DLQ, operator inspection/replay procedure.

Dependencies: M8-03
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Transient fixture retries then succeeds; poison fixture reaches DLQ after exact limit.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Store bounded retry schedule and attempts in PostgreSQL; dead-letter permanent/exhausted work to a durable DLQ; document inspection and replay with audited generations.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
PostgreSQL test verified retry then success and exactly ten failed attempts; RabbitMQ test verified dead-letter routing; strict docs build passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added capped retry schedule, durable attempt history, DLQ policy, operator inspect and audited replay.
<!-- SECTION:FINAL_SUMMARY:END -->
