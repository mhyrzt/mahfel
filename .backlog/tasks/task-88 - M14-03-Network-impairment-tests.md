---
id: TASK-88
title: M14-03 Network impairment tests
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
labels: []
milestone: m-14
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 96000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Latency/loss/retransmission and TURN can dominate user quality.

Implementation scope: Controlled delay/loss/jitter, forced TURN, reconnect, TCP fallback and client/server stats correlation.

Dependencies: M14-01, M3-09
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Repeatable profiles quantify RTT/loss/retransmissions/bitrate and recovery thresholds.
<!-- AC:END -->
