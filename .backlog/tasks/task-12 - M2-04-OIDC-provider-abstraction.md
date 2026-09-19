---
id: TASK-12
title: M2-04 OIDC provider abstraction
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-19 12:14'
labels: []
milestone: m-2
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 20000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Keycloak and future issuers must not leak vendor logic into handlers.

Implementation scope: Discovery, PKCE/state/nonce, callback validation, issuer config, claim normalization, clock/JWK refresh behavior.

Dependencies: M2-01, M2-03
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Fake issuer integration covers success, bad nonce/state/signature, and key rotation.
- [ ] #2 The OIDC callback normalizes claims into AuthenticatedIdentity and calls the identity sync use case, provisioning or resolving the local user by (issuer, subject) and never by email
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Just-in-time provisioning (raised during M1-07 review): the callback must upsert the local user for a valid (issuer, subject) that Mahfel has not seen, so an IdP-only identity is created gracefully on first login. Uses the identity store behind the sync_identity use case from M2-01.
<!-- SECTION:NOTES:END -->
