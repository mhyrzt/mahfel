---
id: TASK-1.5
title: Establish observability foundation
status: Done
assignee: []
created_date: '2026-09-07 23:24'
updated_date: '2026-09-15 02:04'
labels: []
milestone: m-0
dependencies: []
parent_task_id: TASK-1
ordinal: 6000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: services lack consistent structured telemetry and operators cannot scrape or visualize health. Scope: implement JSON logs, spans, request IDs, process/service metrics, readiness state, Prometheus scraping, Grafana provisioning, and starter alert routing.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Prometheus scrapes API, media, and worker operations ports
- [x] #2 Grafana datasource reaches Prometheus
- [x] #3 Metrics avoid identifier labels and other high-cardinality dimensions
- [x] #4 Liveness stays independent of external dependencies
- [x] #5 Readiness reflects each service's critical dependencies
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Follow the approved Mahfel bootstrap plan for this focused slice; update progress and validation evidence through the Backlog CLI.
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented structured tracing, request IDs, low-cardinality process/service metrics, dependency-aware readiness, dependency-independent liveness, Prometheus scraping, alerts, and Grafana provisioning. Live checks showed all three Mahfel targets up and Grafana successfully queried Prometheus.
<!-- SECTION:FINAL_SUMMARY:END -->
