---
id: TASK-11
title: M2-03 Session security
status: Done
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-20 00:09'
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
- [x] #1 Login through Keycloak succeeds and establishes an httpOnly session cookie with secure flags in production
- [x] #2 The BFF forwards only allowlisted /v1 paths with Authorization: Bearer, strips cookies and hop headers, and is not a general proxy
- [x] #3 Middleware protects authenticated routes and redirects unauthenticated users to login
- [x] #4 Expired access tokens refresh through Keycloak; a rejected refresh ends the session
- [x] #5 Logout ends the Keycloak SSO session using id_token_hint
- [x] #6 Frontend lint, typecheck, unit tests, and production build pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Frontend session security (NextAuth + Keycloak, confidential mahfel-web client, httpOnly session cookie, BFF /api/backend/* allowlisted proxy, middleware protection, logout with id_token_hint, refresh). Depends on TASK-12.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Frontend NextAuth v4 + Keycloak confidential mahfel-web (PKCE S256), httpOnly __Secure- cookie, BFF allowlist ^v1/(auth|rooms|echo|status|health), header stripping, 2MiB bound, 10s timeout, middleware getToken with public paths /login/api/auth/api/health, refresh via callbacks.ts, logout with id_token_hint. Typecheck/lint/build pass, 55 backend tests pass. Realm updated to confidential.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Frontend session security implemented: NextAuth + Keycloak, secure session, narrow BFF, protected routes, refresh and logout.
<!-- SECTION:FINAL_SUMMARY:END -->
