---
id: TASK-96
title: M15-05 OpenTelemetry correlation
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
labels: []
milestone: m-15
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 104000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Cross-service latency cannot be reconstructed from unrelated logs.

Implementation scope: W3C trace context over HTTP/WebSocket/gRPC/jobs, OTLP exporter config, sampling, Tempo path and request/job IDs.

Dependencies: M1, M8
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 One classroom control/job flow is traceable end to end with correlated JSON logs.
<!-- AC:END -->
