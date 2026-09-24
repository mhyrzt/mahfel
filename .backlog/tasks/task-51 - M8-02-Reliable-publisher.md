---
id: TASK-51
title: M8-02 Reliable publisher
status: Done
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-24 00:26'
labels: []
milestone: m-8
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 59000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: A successful DB request must not silently lose its job.

Implementation scope: Confirm-mode channel, mandatory routing, timeout/error metrics, stable job/idempotency IDs.

Dependencies: M8-01
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Unroutable/nacked publish is surfaced; confirmed publish integration test passes.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Use a confirm-mode AMQP publisher with mandatory routing, persistent messages, five-second confirm timeout, stable job IDs, and bounded publish outcome metrics. Verify confirmed and unroutable delivery against disposable RabbitMQ.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Confirmed persistent mandatory publish and unroutable error verified in disposable RabbitMQ test; backend lint passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added confirmed publisher, five-second timeout, stable message ID, and bounded outcome metric.
<!-- SECTION:FINAL_SUMMARY:END -->
