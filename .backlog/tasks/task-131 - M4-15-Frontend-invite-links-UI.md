---
id: TASK-131
title: M4-15 Frontend invite links UI
status: In Progress
assignee: []
created_date: '2026-09-20 19:32'
updated_date: '2026-09-25 00:44'
labels:
  - frontend
milestone: m-4
dependencies:
  - TASK-129
  - TASK-130
  - TASK-126
  - TASK-124
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 139000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Browser surfaces for invite links: create, list, copy, revoke, and regenerate a link for a room or a session, and a join-by-code experience that reflects the confirmation state until the request is approved or rejected. Link management is hidden without the manage capability, and the server remains authoritative for every action.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A capable principal can create, copy, list, revoke, and regenerate links for rooms and sessions.
- [ ] #2 The UI shows code, target, expiry, and status, and hides link management without the manage capability.
- [ ] #3 A holder can enter or open a code and sees a waiting or confirmation state until approved or rejected.
- [ ] #4 Expired, revoked, and rejected links show explicit states instead of silent failure.
- [ ] #5 The join-by-code flow works for an authenticated principal and never assumes approval before the server confirms.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
2026-09-25: Added room/session invite management, authenticated code redemption, self-scoped status polling, and manager room-request approval endpoints/UI. Unusable codes remain intentionally non-enumerating, so expired versus revoked cannot be distinguished from the redemption response.
<!-- SECTION:NOTES:END -->
