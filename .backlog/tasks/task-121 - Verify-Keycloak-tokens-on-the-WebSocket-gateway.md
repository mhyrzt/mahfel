---
id: TASK-121
title: Verify Keycloak tokens on the WebSocket gateway
status: Done
assignee:
  - '@mahyar'
created_date: '2026-09-19 19:24'
updated_date: '2026-09-20 00:09'
labels:
  - backend
  - auth
  - websocket
milestone: m-2
dependencies:
  - TASK-12
priority: medium
ordinal: 129000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Replace the interim development authenticator on /v1/ws with the same Keycloak JWT verification used by HTTP, accepting the access token in the auth-first-message. Update the frontend WebSocket client to supply it.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A valid Keycloak access token completes the WebSocket upgrade/authentication; invalid or expired tokens are rejected
- [x] #2 The gateway uses the shared token verifier rather than a second implementation
- [x] #3 Tests cover successful authentication and rejection of an invalid token
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Verify Keycloak tokens on the WebSocket gateway using the shared TokenVerifier (same as HTTP). Support bearer header + auth-first-message, reject unprovisioned tokens, add tests.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented shared TokenVerifier for WebSocket (header + auth-first-message), principal_from_claims with 403 for unprovisioned. Added RealIssuer harness and 6 tests: header valid, header invalid sig (401), message valid, message expired (4401), unprovisioned (4401), auth timeout (4401). All 55 API tests pass. jwks.rs cleaned to use jsonwebtoken::jwk::JwkSet + DecodingKey::from_rsa_components, keeping rotation cache.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
WebSocket gateway now verifies Keycloak tokens via shared verifier; both bearer header and message auth paths tested including expiry, tampering, and unprovisioned cases.
<!-- SECTION:FINAL_SUMMARY:END -->
