---
id: TASK-11
title: M2-03 Session security
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-19 19:31'
labels: []
milestone: m-2
dependencies:
  - TASK-12
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 19000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Browser session security in mahfel-web: NextAuth with the Keycloak provider (confidential client, server-side exchange, PKCE + state + nonce), an httpOnly session cookie with secure flags, token refresh/rotation, route-protecting middleware, and logout. The narrow BFF forwards allowlisted /v1 paths to mahfel-api with the access token (ADR 0019).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Login through Keycloak succeeds and establishes an httpOnly session cookie with secure flags in production
- [ ] #2 The BFF forwards only allowlisted /v1 paths with Authorization: Bearer, strips cookies and hop headers, and is not a general proxy
- [ ] #3 Middleware protects authenticated routes and redirects unauthenticated users to login
- [ ] #4 Expired access tokens refresh through Keycloak; a rejected refresh ends the session
- [ ] #5 Logout ends the Keycloak SSO session using id_token_hint
- [ ] #6 Frontend lint, typecheck, unit tests, and production build pass
<!-- AC:END -->
