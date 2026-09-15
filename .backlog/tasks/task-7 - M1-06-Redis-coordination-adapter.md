---
id: TASK-7
title: M1-06 Redis coordination adapter
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-1
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 15000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Replicas need ephemeral shared coordination without making Redis authoritative.

Implementation scope: Async pool, namespaced keys/TTL, pub-sub strategy, failure classification, health metrics, tests.

Dependencies: M1-01
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Ephemeral key expires; outage behavior is documented/tested; no durable model exists only in Redis.
<!-- AC:END -->
