---
id: TASK-148
title: Restore authenticated API requests after fresh Compose startup
status: Done
assignee:
  - '@codex'
created_date: '2026-09-26 12:53'
updated_date: '2026-09-26 13:15'
labels:
  - backend
  - bug
  - auth
dependencies: []
ordinal: 156000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
After a fresh Compose startup, API token verification returns 503 because OIDC provider metadata is unavailable, and authenticated web pages fail. Public health routes still return 200. Diagnose and repair the API-to-Keycloak issuer discovery path.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Authenticated API requests no longer fail with provider metadata unavailable after a fresh Compose startup
- [x] #2 OIDC issuer and CA validation remain secure and use the public issuer
- [x] #3 Focused checks and live local probe demonstrate the fix
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Capture the underlying discovery failure. 2. Correct the local API-to-Keycloak network or resolver configuration. 3. Add a focused regression check and verify an authenticated API request in the rebuilt stack.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
The user-provided docker compose logs command omitted the mahfel and infrastructure profiles, so Compose excluded the Keycloak dependency; use --profile mahfel --profile infrastructure. During the first startup, web SSR logs showed API 503s and API logs showed OIDC metadata unavailable. The API container was later recreated after the concurrent full Compose build completed. After recreation, health endpoints returned 200, a client-credentials token reached normal API auth rejection (403 rather than metadata-unavailable 503), and API logs recorded successful session.schedule, session.start, websocket connection, and participant.join. Current source already routes the public .localhost issuer through resolved Compose Caddy and mounts its local CA. No lasting source change was needed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The fresh API container now serves authenticated flows; the initial 503s stopped after the API was recreated from the current checkout. Use both Compose profiles when inspecting logs.
<!-- SECTION:FINAL_SUMMARY:END -->
