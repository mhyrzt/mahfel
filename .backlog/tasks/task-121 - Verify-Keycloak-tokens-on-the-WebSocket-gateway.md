---
id: TASK-121
title: Verify Keycloak tokens on the WebSocket gateway
status: To Do
assignee:
  - '@mahyar'
created_date: '2026-09-19 19:24'
updated_date: '2026-09-19 19:31'
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
- [ ] #1 A valid Keycloak access token completes the WebSocket upgrade/authentication; invalid or expired tokens are rejected
- [ ] #2 The gateway uses the shared token verifier rather than a second implementation
- [ ] #3 Tests cover successful authentication and rejection of an invalid token
<!-- AC:END -->
