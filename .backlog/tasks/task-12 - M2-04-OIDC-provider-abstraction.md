---
id: TASK-12
title: M2-04 OIDC provider abstraction
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-19 19:31'
labels: []
milestone: m-2
dependencies:
  - TASK-9
  - TASK-117
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 20000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
mahfel-api is a resource server for Keycloak access tokens (ADR 0019). Add discovery + JWKS verification (issuer, audience, clock skew, key rotation), a JWT-derived Principal, and POST /v1/auth/provision plus GET /v1/auth/me that resolve or create the local user by (issuer, subject). Replace the interim development authenticator. No S3 client work.
<!-- SECTION:DESCRIPTION:END -->




## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Just-in-time provisioning (raised during M1-07 review): the callback must upsert the local user for a valid (issuer, subject) that Mahfel has not seen, so an IdP-only identity is created gracefully on first login. Uses the identity store behind the sync_identity use case from M2-01.
<!-- SECTION:NOTES:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A valid Keycloak access token authenticates; expired, wrong-issuer, or wrong-audience tokens are rejected with 401 and no internal detail
- [ ] #2 POST /v1/auth/provision and GET /v1/auth/me return the local application profile (user id, avatar_url, name, email, username, roles) and never link identities by email
- [ ] #3 First authenticated request provisions the user and identity idempotently; concurrent first requests cannot create duplicates (unique (issuer, subject) with a conflict-safe upsert)
- [ ] #4 The interim development authenticator is removed and authorization stays deny-by-default
- [ ] #5 New endpoints appear in the generated OpenAPI document and the route-coverage test passes
- [ ] #6 Tests cover success, expired token, bad signature, wrong audience, and key rotation against a real Keycloak
<!-- AC:END -->
