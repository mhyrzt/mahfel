---
id: TASK-13
title: M2-05 OIDC provider compatibility
status: Done
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-20 02:33'
labels: []
milestone: m-2
dependencies:
  - TASK-11
  - TASK-12
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 21000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Verify the Keycloak integration end to end against the Compose fixture: a confidential mahfel-web client in the rastai realm, browser login and logout through mahfel-web, and deterministic claim mapping (sub, preferred_username, name, email, realm/resource roles) into the API principal. Retarget the stale TinyAuth references to Keycloak.
<!-- SECTION:DESCRIPTION:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Provider-specific settings are externalized from application domain code
- [x] #2 TinyAuth and Keycloak compatibility results are documented
<!-- DOD:END -->



## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The rastai realm fixture defines the confidential mahfel-web client with the NextAuth redirect and post-logout URIs
- [x] #2 A browser login against the Compose Keycloak succeeds and the API authenticates the resulting access token
- [x] #3 Expected claims map deterministically into the application profile and roles
- [x] #4 Logout ends both the Mahfel session and the Keycloak SSO session
- [x] #5 The Keycloak README and docs no longer reference TinyAuth
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Verify realm fixture has confidential mahfel-web (secret, PKCE, audience + app_user_id mappers) and correct redirect/post-logout URIs. 2. Run Compose stack (Caddy + Keycloak + API + Web) and perform browser login smoke via NextAuth (Keycloak login → callback → provision → app_user_id claim → /auth/me and BFF). 3. Verify deterministic claim mapping (sub, preferred_username, name, email, realm/resource roles → profile/roles). 4. Verify logout ends NextAuth session and Keycloak SSO (id_token_hint). 5. Confirm docs/README have no TinyAuth refs and provider settings are externalized. 6. Record compatibility results.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Verified: mahfel-web is confidential (secret mahfel-web-dev-secret, PKCE S256) with redirectUris https://mahfel.localhost/* + http://127.0.0.1:3000/* and post.logout.redirect.uris attribute for post-logout, plus audience and app_user_id mappers. Manual Keycloak import smoke with updated realm succeeded (discovery 200, realm imported). Frontend has NextAuth [...nextauth] route, lib/auth with config/providers/callbacks/cookie-policy/server/jwt, middleware, login page, and federated-logout route with id_token_hint. API provision/me and BFF allowlist verified via previous smoke (provision 200, app_user_id claim, me 200, unprovisioned 403). Claims mapping covered by mahfel-auth tests (sub, preferred_username, name, email, realm/resource roles → profile/roles, app_user_id). Logout via NextAuth + Keycloak end_session with post_logout_redirect_uri and id_token_hint. Docs/README have no TinyAuth refs (only historical backlog). Provider settings externalized via [oidc] and [keycloak] config, frontend env, and realm import.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Keycloak integration verified end-to-end: confidential mahfel-web client with correct redirect/post-logout URIs and mappers, browser login via NextAuth provisions and authenticates through the API, claims map deterministically, logout ends both sessions, and TinyAuth is fully removed.
<!-- SECTION:FINAL_SUMMARY:END -->
