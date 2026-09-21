---
id: TASK-124
title: M4-04 Session capabilities and authorization
status: To Do
assignee: []
created_date: '2026-09-20 18:58'
labels:
  - backend
  - security
milestone: m-4
dependencies:
  - TASK-15
  - TASK-114
  - TASK-26
documentation:
  - docs/backlog/roadmap.md
priority: high
ordinal: 132000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Decide every session and moderation action from capabilities derived from verified global roles and per-room membership, instead of a fixed vertical role enum. Define a capability set (join, publish_media, share_screen, mute_others, remove_participant, admit_participant, manage_session) and one policy seam that maps global admin/moderator, room manager/member, and service principals to capabilities deny-by-default, reused by REST and WebSocket.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A single policy resolves a principal to capabilities from verified claims plus room membership, and no handler hardcodes role strings.
- [ ] #2 Deny-by-default allow/deny matrix tests cover member, manager, moderator, admin, and a service principal.
- [ ] #3 REST and WebSocket session actions evaluate capabilities through the same seam.
- [ ] #4 The caller can introspect its own session capabilities without exposing other participants privilege levels.
- [ ] #5 Capability names and meanings are documented, including the education mapping of teacher to manager or host.
<!-- AC:END -->
