---
id: TASK-19
title: M3-03 WebRtcServer/transports
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-3
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 27000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Browsers need reachable ICE/DTLS transports on predictable ports.

Implementation scope: One WebRtcServer per worker, UDP/TCP listen infos, announced address, send/receive transport lifecycle, ICE/DTLS parameters.

Dependencies: M3-02
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Browser transport connects over direct UDP; TCP fallback behavior is recorded.
<!-- AC:END -->
