---
id: TASK-118
title: API resource-server authentication and identity provisioning
status: To Do
assignee:
  - '@mahyar'
created_date: '2026-09-19 19:24'
labels:
  - backend
  - auth
milestone: m-2
dependencies:
  - TASK-117
  - TASK-9
priority: high
ordinal: 126000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Make mahfel-api a resource server for Keycloak access tokens: discovery + JWKS verification (issuer, audience, clock skew, key rotation), a JWT-derived Principal, and provision/me endpoints that resolve or create the local user by (issuer, subject). Replaces the interim development authenticator. No S3 client work.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A valid Keycloak access token authenticates; expired, wrong-issuer, or wrong-audience tokens are rejected with 401 and no internal detail
- [ ] #2 POST /v1/auth/provision and GET /v1/auth/me return the local application profile (user id, avatar_url, name, email, username, roles) and never link identities by email
- [ ] #3 First authenticated request provisions the user and identity idempotently; concurrent first requests cannot create duplicates (unique (issuer, subject) with a conflict-safe upsert)
- [ ] #4 The interim development authenticator is removed and authorization stays deny-by-default
- [ ] #5 New endpoints appear in the generated OpenAPI document and the route-coverage test passes
- [ ] #6 Tests cover success, expired token, bad signature, wrong audience, and key rotation against a real Keycloak
<!-- AC:END -->
