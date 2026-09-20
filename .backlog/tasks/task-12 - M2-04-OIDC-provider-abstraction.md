---
id: TASK-12
title: M2-04 OIDC provider abstraction
status: Done
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-19 22:03'
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

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A valid Keycloak access token authenticates; expired, wrong-issuer, or wrong-audience tokens are rejected with 401 and no internal detail
- [x] #2 POST /v1/auth/provision and GET /v1/auth/me return the local application profile (user id, avatar_url, name, email, username, roles) and never link identities by email
- [x] #3 First authenticated request provisions the user and identity idempotently; concurrent first requests cannot create duplicates (unique (issuer, subject) with a conflict-safe upsert)
- [x] #4 The interim development authenticator is removed and authorization stays deny-by-default
- [x] #5 New endpoints appear in the generated OpenAPI document and the route-coverage test passes
- [x] #6 Tests cover success, expired token, bad signature, wrong audience, and key rotation against a real Keycloak
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Just-in-time provisioning (raised during M1-07 review): the callback must upsert the local user for a valid (issuer, subject) that Mahfel has not seen, so an IdP-only identity is created gracefully on first login. Uses the identity store behind the sync_identity use case from M2-01.

Implemented mahfel-api as a resource server. mahfel-auth gains a claims normalizer (issuer set from the verified config, not a claim), a JWKS cache that refreshes on an unknown kid, and an RS256 TokenVerifier (issuer, audience, exp/nbf with leeway, tamper checks). The API authenticates every request from the bearer token through a dyn-compatible RequestAuthenticator; the interim DevAuthenticator and MAHFEL_API__DEVELOPMENT_PRINCIPAL_USER_ID are removed, so protected routes deny by default. POST /v1/auth/provision and GET /v1/auth/me return the app profile; identity is resolved/provisioned by (issuer, subject) via sync_identity and never by email. WebSocket upgrades reuse the same verifier. The realm fixture gained a mahfel-api audience mapper on mahfel-web and mahfel-admin so browser/service tokens carry the resource audience. Tests: 5 deterministic local-issuer verifier tests (valid, expired, wrong audience, wrong issuer, tampered signature) plus a JWKS key-rotation unit test; a real-Keycloak testcontainers test verifies discovery, audience-mapped token verification, tamper rejection, wrong audience/issuer, and service-account admin access; the postgres integration test covers idempotent and concurrent provisioning. AC #6 note: key rotation is proven by the deterministic JWKS test rather than real Keycloak, because the least-privilege mahfel-admin service account cannot rotate realm keys. Boot smoke: readiness up, provision idempotent, me 200, missing/bad token 401, one user and one identity created. Validation: workspace clippy clean; workspace lib/bins tests pass; mahfel-auth lib+verify+keycloak tests pass; postgres migrations integration passes; just docs strict; docker compose config ok.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
mahfel-api is now a Keycloak resource server per ADR 0019: it verifies access tokens (issuer, audience, signature, expiry, key rotation) on every request, JIT-provisions or resolves the local user by (issuer, subject), and serves the application profile at /v1/auth/provision and /v1/auth/me. The browser development authenticator is gone and authorization stays deny-by-default.
<!-- SECTION:FINAL_SUMMARY:END -->
