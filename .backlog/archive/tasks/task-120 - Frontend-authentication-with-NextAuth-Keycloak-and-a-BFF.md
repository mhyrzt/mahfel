---
id: TASK-120
title: 'Frontend authentication with NextAuth, Keycloak, and a BFF'
status: To Do
assignee:
  - '@mahyar'
created_date: '2026-09-19 19:24'
updated_date: '2026-09-19 19:31'
labels:
  - frontend
  - auth
milestone: m-2
dependencies: []
priority: high
ordinal: 128000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement browser authentication in mahfel-web: NextAuth with the Keycloak provider (confidential client, server-side exchange, PKCE + state), a narrow BFF that proxies /api/backend/* to mahfel-api with the access token, middleware route protection, login and Keycloak logout, and a typed API client. Update the realm fixture with the confidential mahfel-web client.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Login through Keycloak succeeds and establishes an httpOnly session cookie with secure flags in production
- [ ] #2 The BFF forwards only allowlisted /v1 paths with Authorization: Bearer, strips cookies and hop headers, and is not a general proxy
- [ ] #3 Middleware protects authenticated routes and redirects unauthenticated users to login
- [ ] #4 Logout ends the Keycloak SSO session using id_token_hint
- [ ] #5 The session exposes the application profile and avatar_url returned by the API
- [ ] #6 Frontend lint, typecheck, unit tests, and production build pass
<!-- AC:END -->
