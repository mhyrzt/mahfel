---
id: TASK-55
title: M8-06 Transactional outbox
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
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
- [ ] #1 DB change creates durable job atomically; broker outage later drains without loss/duplication effect.
<!-- AC:END -->
