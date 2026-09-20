---
id: TASK-17
title: M3-01 Worker supervision
status: Done
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-20 10:11'
labels: []
milestone: m-3
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 25000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Native worker crashes/resource limits need explicit lifecycle behavior.

Implementation scope: Configurable worker count/settings, spawn supervision, failure metrics, readiness, shutdown and restart policy.

Dependencies: M0
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 At least one worker is required for readiness; induced failure is detected and documented.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Inspect current MediaRuntime and media main for supervision gaps\n2. Enhance mahfel-rtc: add worker event monitoring, restart policy, failure metrics, readiness semantics (require >=1 active worker)\n3. Expose worker health via telemetry crate and /metrics\n4. Add shutdown/restart handling and documented failure behavior\n5. Validate with unit tests for zero workers, insufficient ports, and worker closed detection
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented M3-01 worker supervision: validated worker_count>0, added WebRtcServer per worker with announced_address, exposed mahfel_media_active_workers gauge and mahfel_media_worker_failures_total counter on /metrics (9102), made readiness require active_workers>0 with 5s background probe that detects closed workers, logs warning and increments failure metric, documented restart policy (no in-process respawn; orchestrator restarts container) in mahfel-rtc docs and media main comments.
<!-- SECTION:NOTES:END -->
