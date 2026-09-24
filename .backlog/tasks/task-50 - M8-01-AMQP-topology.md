---
id: TASK-50
title: M8-01 AMQP topology
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
ordinal: 58000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Jobs need durable versioned routing and ownership.

Implementation scope: Exchanges, queues, routing keys, durable declarations, prefetch, schema envelope/version and compatibility rules.

Dependencies: M0, M1
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Topology declaration is idempotent; compatible schema fixture round-trips.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Declare durable versioned job exchange, file cleanup queue, DLX and DLQ; define typed v1 envelope and compatibility fixtures; verify idempotent declarations against disposable RabbitMQ.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
RabbitMQ topology declared twice against a disposable broker; v1 compatibility fixture round-trips.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Defined versioned envelope and durable RabbitMQ topology with quorum queue and DLQ. Verified by application unit tests and RabbitMQ integration test.
<!-- SECTION:FINAL_SUMMARY:END -->
