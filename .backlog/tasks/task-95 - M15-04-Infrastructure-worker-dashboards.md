---
id: TASK-95
title: M15-04 Infrastructure/worker dashboards
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
labels: []
milestone: m-15
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 103000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Queue backlog and dependency saturation need actionable views.

Implementation scope: Rabbit depth/rates/unacked/DLQ, worker jobs/failures, Redis, Postgres pool/query, S3 latency/errors and coturn.

Dependencies: M6, M8, M13
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Operator diagnoses queue backlog, worker failure, PostgreSQL saturation, or S3 errors.
<!-- AC:END -->
