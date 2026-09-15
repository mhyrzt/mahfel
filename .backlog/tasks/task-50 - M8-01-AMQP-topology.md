---
id: TASK-50
title: M8-01 AMQP topology
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
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
- [ ] #1 Topology declaration is idempotent; compatible schema fixture round-trips.
<!-- AC:END -->
