---
id: TASK-122
title: LMS-style provisioning with an app_user_id claim
status: Done
assignee:
  - '@mahyar'
created_date: '2026-09-19 22:53'
updated_date: '2026-09-19 23:37'
labels:
  - backend
  - auth
milestone: m-2
dependencies:
  - TASK-12
priority: high
ordinal: 130000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Switch from per-request identity lookup to the LMS pattern: normal requests trust an app_user_id claim that Keycloak injects from a user attribute, and only POST /v1/auth/provision writes identity. Add a mahfel-keycloak admin adapter (client credentials) that links the Keycloak user to the local user id, add the app_user_id claim to the realm fixture, and stop resolving the local user on every request.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The rastai realm fixture maps the mahfel_user_id user attribute to an app_user_id access-token claim on the browser client
- [x] #2 A mahfel-keycloak admin adapter authenticates with the mahfel-admin client-credentials grant and sets a user attribute idempotently, with typed failure classification
- [x] #3 POST /v1/auth/provision verifies the token, provisions or refreshes the local user by (issuer, subject), writes the local user id to Keycloak, and returns the application profile
- [x] #4 Normal authenticated requests resolve the principal from the app_user_id claim and perform no identity database lookup
- [x] #5 A verified token without app_user_id is rejected as not provisioned, while /v1/auth/provision remains reachable before provisioning
- [x] #6 Tests cover claim parsing, the admin adapter against a disposable Keycloak, and the pre/post provisioning request paths
- [x] #7 ADR 0019 and the backend AGENTS documentation describe the claim-based model and the frontend token-refresh requirement
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. mahfel-auth: parse app_user_id from claims into AuthenticatedIdentity.
2. New mahfel-keycloak crate: client-credentials admin adapter with a cached token and set_user_attribute, typed errors.
3. API auth: build Principal from the app_user_id claim (no DB); add a verified-token extractor for provision; reject unprovisioned tokens as forbidden.
4. API identity routes: provision synchronizes the user and links it in Keycloak; me reads the claim-derived profile.
5. Wire [keycloak] config and the adapter into AppState and the composition root.
6. Tests: claim parsing, admin adapter against disposable Keycloak, and pre/post provisioning API paths.
7. Docs: ADR 0019 note, backend AGENTS, Keycloak README; validate clippy/tests/smoke.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented the LMS-style model. Realm fixture: both clients map the mahfel_user_id user attribute to an app_user_id access-token claim via an oidc-usermodel-attribute-mapper. New mahfel-keycloak crate: a client-credentials admin adapter with a cached token that sets a user attribute idempotently and classifies transport vs rejected-service-account failures. API: RequestAuthenticator now returns verified claims, principal_from_claims requires app_user_id (403 when absent), and normal routes perform no identity lookup; a VerifiedToken extractor keeps POST /v1/auth/provision reachable before provisioning, where it syncs the user by (issuer, subject) and writes the linking attribute. Refactored per review: http/auth is now a folder (verification/extract/policy/principal) and the outbound ports moved to mahfel-application (ports::UserAttributeWriter with AttributeError, users::IdentityProvisioner), with mahfel-keycloak implementing the attribute port. Tests: claim parsing incl. app_user_id; three API tests for provisioned, unprovisioned (403) and provision paths; two mahfel-keycloak testcontainers tests proving set/replace idempotency and rejected-service-account classification. End-to-end smoke: /v1/auth/me 403 before provisioning, provision 200 creates one user+identity, refreshed token carries app_user_id, /v1/auth/me then 200. Validation: workspace clippy clean; workspace lib/bins tests pass; mahfel-auth lib+verify+keycloak tests pass; mahfel-keycloak admin tests pass; just docs strict; docker compose config ok. Docs updated: ADR 0019, backend AGENTS, architecture components and data-flow, Keycloak README, testing.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Mahfel now provisions like the LMS: Keycloak is the login authority, POST /v1/auth/provision performs the one-time identity sync and links the local user id onto the Keycloak identity, and every other request builds its principal from the app_user_id claim with no identity database lookup. An unprovisioned token is rejected with 403 until the client refreshes after provisioning. The auth transport was split into focused modules and its outbound ports moved into mahfel-application.
<!-- SECTION:FINAL_SUMMARY:END -->
