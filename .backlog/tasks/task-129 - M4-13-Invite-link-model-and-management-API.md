---
id: TASK-129
title: M4-13 Invite link model and management API
status: To Do
assignee: []
created_date: '2026-09-20 19:31'
labels:
  - backend
  - api
  - security
milestone: m-4
dependencies:
  - TASK-26
  - TASK-124
documentation:
  - docs/backlog/roadmap.md
priority: high
ordinal: 137000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Introduce revocable, human-typable invite links for rooms and sessions, in the spirit of a Google Meet link. A link has a globally unique code in the form xxx-yyyy-zzz (lowercase a-z), a target (one room or one session), an expiry defaulting to 7 days and bounded by policy, a status of active, revoked, or expired, and a creator. Management API supports create, list, revoke, rotate, and regenerate for a room or session, gated by the manage capability. Codes are stored plaintext because they must be looked up directly, and are treated as bearer capabilities with rate-limited redemption, audit, and strict log hygiene.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Codes match the xxx-yyyy-zzz format, are globally unique, and are generated atomically with bounded collision retry; redemption normalizes mixed case and separators.
- [ ] #2 A link targets exactly one room or one session and stores expiry, status, and creator; expired or revoked links are rejected on redemption.
- [ ] #3 Create, list, revoke, rotate, and regenerate require the manage capability for the target and are deny-by-default.
- [ ] #4 Rotate and regenerate invalidate the previous code atomically, and revoke is terminal.
- [ ] #5 Redemption is rate-limited per principal and address, create and revoke and redeem emit audit events, and codes never appear in logs or metrics labels.
<!-- AC:END -->
