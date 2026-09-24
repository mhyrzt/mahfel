---
id: TASK-52
title: M8-03 Consumer acknowledgements
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
- [x] #1 Successful job acked; shutdown stops intake and finishes/returns in-flight jobs safely.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Consume with manual ack and bounded prefetch/concurrency; classify handler results, acknowledge only after PostgreSQL completion or retry is durable, stop intake and drain active work on shutdown.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Worker end-to-end test verified manual success acknowledgement and clean shutdown; concurrency and prefetch are configured.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added bounded manual-ack consumer with typed retryable and permanent outcomes, claim renewal, and graceful draining.
<!-- SECTION:FINAL_SUMMARY:END -->
