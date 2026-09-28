---
id: TASK-146
title: Restore RFC 3339 API timestamps for room and session flows
status: In Progress
assignee:
  - '@codex'
created_date: '2026-09-26 12:08'
updated_date: '2026-09-26 12:16'
labels:
  - frontend
  - backend
  - bug
dependencies: []
ordinal: 154000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Authenticated room and workspace views crash with Invalid time value because API timestamp DTOs serialize as tuples while the browser contract expects date-time strings. Audit adjacent room/session responses and requests for the same mismatch.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Room and session API responses emit RFC 3339 timestamp strings
- [x] #2 Session scheduling requests accept RFC 3339 timestamps from the browser
- [ ] #3 Room and workspace views render seeded sessions without invalid date errors
- [x] #4 Regression tests cover the serialized API contract and relevant frontend date rendering
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Audit affected DTO fields and existing RFC 3339 helper. 2. Apply the helper to the affected response and request timestamps. 3. Add focused serialization and rendering regression tests. 4. Run affected backend/frontend gates and verify the live local stack where available.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Applied the existing RFC 3339 serde helper to session, file, and chat response timestamps and session create/update input. API unit suite: 149 passed; strict API Clippy and formatting passed. Frontend room tests: 7 passed; typecheck and lint passed. Rebuilt and restarted only mahfel-api in local Compose; container healthy and /v1/status returned 200. Authenticated room click-through remains unverified because the in-app browser rejects the local development certificate; it was not bypassed.
<!-- SECTION:NOTES:END -->
