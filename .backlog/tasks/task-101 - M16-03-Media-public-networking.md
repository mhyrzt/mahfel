---
id: TASK-101
title: M16-03 Media public networking
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
labels: []
milestone: m-16
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 109000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: SFU candidates fail if stable addresses or port preservation are wrong.

Implementation scope: Per-replica stable IP decision, 40000+ UDP/TCP service, announced address injection, firewall and flow affinity.

Dependencies: M16-01, platform answers
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Two nodes receive reachable candidates; external/container ports match; direct UDP verified.
<!-- AC:END -->
