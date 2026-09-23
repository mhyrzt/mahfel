---
id: TASK-34
title: M5-03 Realtime fanout
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
ordinal: 42000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Multiple API replicas must deliver the same committed events.

Implementation scope: Transaction-to-event boundary, Redis pub-sub/stream decision, local connection hub, sequence/dedupe and backpressure.

Dependencies: M1-05, M1-06, M5-01
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Clients on two API replicas receive committed message once; slow client is bounded.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Persist ordered room chat events atomically with message mutations. 2. Add room WebSocket subscriptions with authorized catch-up, deduplication, and bounded delivery. 3. Use Redis pub/sub for prompt cross-replica wakeups and PostgreSQL polling for recovery. 4. Test two replicas, missed wakeups, mutation events, authorization, and slow clients; update contracts and architecture docs.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
PostgreSQL event log, Redis wakeups, authorized room subscriptions, replay/dedupe, and bounded WebSocket queues. Validation: 359 unit and 380 integration tests passed; two-registry dedupe/backpressure test and PostgreSQL event tests passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented durable ordered chat events and cross-replica fanout with Redis acceleration and PostgreSQL catch-up.
<!-- SECTION:FINAL_SUMMARY:END -->
