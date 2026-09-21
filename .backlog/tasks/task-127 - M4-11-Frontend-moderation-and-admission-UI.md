---
id: TASK-127
title: M4-11 Frontend moderation and admission UI
status: To Do
assignee: []
created_date: '2026-09-20 18:58'
updated_date: '2026-09-20 19:00'
labels:
  - frontend
milestone: m-4
dependencies:
  - TASK-126
  - TASK-124
  - TASK-125
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 135000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Capability-gated moderation and admission controls in the browser app: mute request, enforced mute, remove, block, and publish or screen gating, plus an admission queue with admit and reject. The server stays authoritative and the UI only reflects capability and server state.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Moderation controls appear only with the matching capability and the server still rejects unauthorized actions.
- [ ] #2 An enforced mute is reflected for the affected participant.
- [ ] #3 The admission queue lists waiting principals with admit and reject and updates live.
- [ ] #4 Removed or blocked participants see an explicit state instead of a silent failure.
- [ ] #5 Capability-gated actions give clear feedback and never assume success before the server confirms.
<!-- AC:END -->
