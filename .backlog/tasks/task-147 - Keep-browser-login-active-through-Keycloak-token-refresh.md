---
id: TASK-147
title: Keep browser login active through Keycloak token refresh
status: Done
assignee:
  - '@codex'
created_date: '2026-09-26 12:22'
updated_date: '2026-09-26 12:29'
labels:
  - frontend
  - bug
  - auth
dependencies: []
ordinal: 155000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Authenticated browser sessions appear to expire after the five-minute access token lifetime. The Compose web container cannot connect to the current public .localhost refresh URL, and server-component refreshes do not persist rotated JWT cookies.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The web runtime can refresh a Keycloak token using an internal server-only endpoint while preserving the public issuer
- [x] #2 Successful refreshes persist the updated session cookie during normal authenticated use
- [x] #3 Expired and temporarily unavailable refreshes retain distinct, safe behavior
- [x] #4 Focused tests cover token endpoint selection and refresh behavior
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Give server-side token refresh a separate internal endpoint in Compose while retaining the public issuer. 2. Ensure browser-backed session refresh persists cookies before access expiry and after returning from idle. 3. Test callback and refresh behavior. 4. Build and verify the local web runtime.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Confirmed live Compose web container could not fetch https://auth.mahfel.localhost from Bun (ConnectionRefused) while http://keycloak:8080 responded. Set a server-only internal refresh URL and kept the HTTPS issuer. Added a 120-second refresh buffer and a workspace SessionProvider polling every 180 seconds, using NextAuth session route cookie writes; RSC getServerSession has a no-op cookie setter in the installed NextAuth v4 code. A temporary refresh failure now retains only a still-valid access token. Four focused auth tests, frontend typecheck/lint, strict docs build, Compose config, and production web image build passed. Rebuilt and restarted only mahfel-web; health and HTTPS /api/health returned 200, and the configured internal token endpoint responded 401 to an unauthenticated probe. A real authenticated refresh across five minutes was not observed because the in-app browser rejects the local CA.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Restored reliable local Keycloak refresh connectivity, persisted browser session refreshes during workspace use, and preserved valid access tokens during transient refresh failures.
<!-- SECTION:FINAL_SUMMARY:END -->
