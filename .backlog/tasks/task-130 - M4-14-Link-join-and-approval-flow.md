---
id: TASK-130
title: M4-14 Link join and approval flow
status: To Do
assignee: []
created_date: '2026-09-20 19:31'
updated_date: '2026-09-20 19:32'
labels:
  - backend
  - api
  - security
milestone: m-4
dependencies:
  - TASK-129
  - TASK-125
  - TASK-124
  - TASK-27
documentation:
  - docs/backlog/roadmap.md
priority: high
ordinal: 138000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Redeem an invite link and drive the confirmation flow. An authenticated principal that redeems a valid link enters a waiting state unless it already holds explicit membership in the target. A principal with the admit capability approves or rejects the request. Approval grants session participation for a session link, and durable room membership for a room link, after which future joins bypass confirmation. Reject terminates the waiting state. This reuses the admission seam and the capability seam rather than adding a parallel mechanism.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Redeeming a valid link returns the resolved target and a waiting state unless the principal is already an explicit member of the target.
- [ ] #2 Approving a session link admits the principal to that session only and creates no durable room membership.
- [ ] #3 Approving a room link creates durable room membership, and subsequent joins to that room bypass confirmation.
- [ ] #4 Approve and reject require the admit capability, reject ends the waiting state, and no media or publish capability is granted before approval.
- [ ] #5 Invalid, expired, or revoked codes and repeated failed attempts return deliberate non-enumerating errors, are rate-limited, and are audited.
<!-- AC:END -->
