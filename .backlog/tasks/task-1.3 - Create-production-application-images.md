---
id: TASK-1.3
title: Create production application images
status: Done
assignee: []
created_date: '2026-09-07 23:24'
updated_date: '2026-09-15 02:04'
labels: []
milestone: m-0
dependencies: []
parent_task_id: TASK-1
ordinal: 4000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: none of the four applications can be built or deployed as containers. Scope: create cache-aware multi-stage Dockerfiles with non-root runtime users, health checks, graceful signals, and native mediasoup build prerequisites.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Each of the four application images builds
- [x] #2 Compiler and native build dependencies are absent from runtime stages
- [x] #3 Runtime images use non-root users and expose only documented ports
- [x] #4 Health checks target the dedicated service health endpoints
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Follow the approved Mahfel bootstrap plan for this focused slice; update progress and validation evidence through the Backlog CLI.
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Built all four production multi-stage images. Runtime stages exclude compilers, run as uid 10001, expose documented ports, and use service health endpoints. The media builder pins Rust 1.95 and installs native mediasoup prerequisites; its validation target runs all Rust gates.
<!-- SECTION:FINAL_SUMMARY:END -->
