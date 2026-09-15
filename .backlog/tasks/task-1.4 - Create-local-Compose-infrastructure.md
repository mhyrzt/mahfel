---
id: TASK-1.4
title: Create local Compose infrastructure
status: Done
assignee: []
created_date: '2026-09-07 23:24'
updated_date: '2026-09-15 02:04'
labels: []
milestone: m-0
dependencies: []
parent_task_id: TASK-1
ordinal: 5000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: contributors cannot run Mahfel with its required dependencies. Scope: compose the four applications with PostgreSQL, Redis, RabbitMQ, MinIO, coturn, Prometheus, Grafana, Alertmanager, networks, volumes, health conditions, and safe development configuration.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 docker compose config validates
- [x] #2 just compose-up starts the complete stack
- [x] #3 Infrastructure health checks become healthy
- [x] #4 Application services are stateless and management ports are loopback/private
- [x] #5 RTC and TURN port ranges are documented and configurable
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Follow the approved Mahfel bootstrap plan for this focused slice; update progress and validation evidence through the Backlog CLI.
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Validated and live-tested the complete Compose stack with PostgreSQL, Redis, RabbitMQ, MinIO, coturn, Prometheus, Alertmanager, Grafana, and four stateless applications. Private/admin host bindings are loopback-only on a dedicated admin bridge; host ports and RTC/TURN ranges are configurable.
<!-- SECTION:FINAL_SUMMARY:END -->
