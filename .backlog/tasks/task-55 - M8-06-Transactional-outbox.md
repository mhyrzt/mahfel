---
id: TASK-55
title: M8-06 Transactional outbox
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
ordinal: 63000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: DB commit and broker publish cannot be one transaction.

Implementation scope: Outbox table/write with domain transaction, relay leasing, confirms, retry, cleanup and lag metrics.

Dependencies: M8-02, M8-05
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 DB change creates durable job atomically; broker outage later drains without loss/duplication effect.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Write job and outbox inside the chat deletion transaction; lease and confirm relay publication, recover broker outages, prune confirmed rows, and expose lag metrics.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Worker end-to-end test committed chat deletion before RabbitMQ startup, drained the outbox, cleaned object and metadata, and recorded success.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added transactional chat cleanup outbox, confirmed relay, publish recovery, pruning, and lag metrics.
<!-- SECTION:FINAL_SUMMARY:END -->
