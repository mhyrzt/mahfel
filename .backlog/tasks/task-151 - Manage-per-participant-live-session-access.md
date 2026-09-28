---
id: TASK-151
title: Manage per-participant live session access
status: In Progress
assignee: []
created_date: '2026-09-27 04:41'
updated_date: '2026-09-27 07:42'
labels:
  - backend
  - frontend
  - bug
dependencies: []
ordinal: 159000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Let room managers grant or revoke talk, camera, screen share, presentation, and editing independently for admitted session participants. Managers cannot change their own or another manager access. Enforce changes in API/media and collaboration operations.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A manager can change each access flag independently for another admitted member
- [ ] #2 A manager cannot change their own access or another manager access
- [ ] #3 Revoked media access blocks new publication and closes an active producer
- [ ] #4 Presentation and document editing are authorized independently
- [ ] #5 The live UI reflects current access and the contract and migration are documented
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented SQL-backed per-participant audio, video, screen, presentation, and editing permissions with manager/self denial, REST and WebSocket enforcement, active media closure, UI controls, generated OpenAPI, docs, and migration integration coverage. Local cargo check, backend lint, database migration test, frontend lint/typecheck/unit tests, and docs passed. Live two-account media and access behavior remains unverified on the undeployed build.
<!-- SECTION:NOTES:END -->
