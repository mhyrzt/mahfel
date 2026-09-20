---
id: TASK-119
title: Seed Keycloak users and mirror them into Postgres
status: To Do
assignee:
  - '@mahyar'
created_date: '2026-09-19 19:24'
updated_date: '2026-09-19 23:54'
labels:
  - backend
  - auth
  - seed
milestone: m-2
dependencies:
  - TASK-9
  - TASK-117
  - TASK-12
priority: high
ordinal: 127000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add a Keycloak Admin adapter that authenticates with the client-credentials grant of the mahfel-admin service account (no interactive administrator credentials), and make mahfel-api seed upsert the development identities into Keycloak and then mirror users and identities into Postgres. Add a nullable users.avatar_url column (S3 link) with no S3 client yet.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The Keycloak admin adapter authenticates with client credentials and is the only path mahfel-api uses to manage the realm
- [ ] #2 seed creates missing development users in Keycloak and leaves existing users untouched; it never resets an existing password
- [ ] #3 seed mirrors every fixture user into users and identities idempotently, including users that already existed in Keycloak
- [ ] #4 A nullable users.avatar_url column exists via a forward migration and is returned by the profile endpoints
- [ ] #5 The realm import no longer hardcodes development users; the seed is the single source of fixture identities
- [ ] #6 Integration tests run against a disposable Keycloak and prove create-if-missing and mirror idempotency
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Postponed per 2026-09-20 review: user has a separate plan for seeds. Keep To Do for later; focus now is TASK-11 (frontend session/BFF), TASK-121 (WebSocket verification) and jwks.rs cleanup.
<!-- SECTION:NOTES:END -->
