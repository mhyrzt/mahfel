---
id: TASK-9
title: M2-01 Identity schema
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-19 12:13'
labels: []
milestone: m-2
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 17000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: One Mahfel user may have multiple login identities.

Implementation scope: Users, identities (issuer, subject), verified emails, avatar source, local credential table, uniqueness/migrations.

Dependencies: M1
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Constraint tests prevent identity takeover and allow multiple identities per user.
- [ ] #2 First login for an unseen (issuer, subject) provisions a local user idempotently; concurrent first logins cannot create duplicates (unique (issuer, subject) enforced with a conflict-safe upsert)
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Just-in-time provisioning requirement (raised during M1-07 review): an identity may authenticate at the IdP (Dex/Keycloak/Zitadel) before it exists in mahfel.users. The identities table plus a subject-keyed, conflict-safe upsert must create the local user on first login. Identity is keyed on (issuer, subject); email must never be a linking key. The sync_identity application slice is currently a non-persisting placeholder and must be completed against this schema.
<!-- SECTION:NOTES:END -->
