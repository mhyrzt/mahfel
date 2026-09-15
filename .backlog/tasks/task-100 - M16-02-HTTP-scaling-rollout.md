---
id: TASK-100
title: M16-02 HTTP scaling rollout
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
labels: []
milestone: m-16
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 108000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Web/API/worker need safe horizontally scaled release behavior.

Implementation scope: Ingress domains/TLS/CORS, API replicas, worker concurrency, resource limits/autoscaling, migration job and rollback.

Dependencies: M16-01
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Two API and two worker replicas pass functional/rolling deployment tests.
<!-- AC:END -->
