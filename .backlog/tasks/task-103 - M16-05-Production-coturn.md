---
id: TASK-103
title: M16-05 Production coturn
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
updated_date: '2026-10-03 11:28'
labels: []
milestone: m-17
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 111000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Local static credentials/no TLS are unsafe for production.

Implementation scope: Stable public IP/DNS, TLS 5349 and optional 443, time-limited credentials, relay range, bandwidth/resources and monitoring.

Dependencies: M16-01, M13-07
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Forced TURN works with valid TLS/auth and relay load stays within measured limits.
<!-- AC:END -->
