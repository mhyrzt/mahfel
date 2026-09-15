---
id: TASK-98
title: M15-07 Error/AI usage telemetry
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
labels: []
milestone: m-15
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 106000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Exceptions and provider spend need bounded privacy-aware visibility.

Implementation scope: Sentry integration boundary, scrub rules, AI token/cost/latency/error aggregates, quotas and provider status.

Dependencies: M11, M12
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Test error correlates without prompt/audio/secret leakage; aggregate AI usage matches fixtures.
<!-- AC:END -->
