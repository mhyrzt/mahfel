---
id: TASK-116
title: Replace Dex with Keycloak as the local OIDC provider
status: Done
assignee:
  - '@mahyar'
created_date: '2026-09-19 18:47'
updated_date: '2026-09-19 19:10'
labels:
  - infrastructure
  - auth
milestone: m-2
dependencies: []
references:
  - docker-compose.yml
  - deploy/caddy/Caddyfile
  - backend/config/default.toml
  - backend/crates/auth
priority: high
ordinal: 124000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: local Compose runs Dex while production runs Keycloak, so the auth path is validated against a provider we do not deploy. One provider reduces drift and lets M2-04/M2-05 be developed and verified against the real compatibility target.

Implementation scope: swap the Compose service, Caddy route, and issuer configuration to Keycloak; add a committed realm import with the API client and permanent development identities (password 'password', stable subject IDs, realm roles); replace the Dex discovery integration test with a Keycloak one; update architecture/deployment docs and record an ADR. The OIDC client stays in mahfel-api (API-owned callback).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Compose runs Keycloak instead of Dex and 'docker compose config' is valid; discovery is served at https://auth.mahfel.localhost/realms/mahfel
- [x] #2 A committed realm import defines the mahfel realm, the confidential API client (redirect https://api.mahfel.localhost/v1/auth/oidc/callback), and the permanent development identities with password 'password' and stable subject IDs
- [x] #3 Caddy proxies auth.mahfel.localhost to Keycloak with forwarded headers so the issuer matches for browser and container callers
- [x] #4 backend config defaults and .env.example point at the Keycloak issuer and no Dex-specific settings remain
- [x] #5 The mahfel-auth provider integration test targets Keycloak (gated by MAHFEL_TESTCONTAINERS) and claim normalization covers Keycloak realm/resource roles
- [x] #6 Architecture and local deployment docs plus an ADR record Keycloak as the single OIDC provider; Dex references are removed
- [x] #7 Validation: docker compose config, Keycloak discovery reachable, mahfel-auth tests, and just docs strict
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Compose: replace the dex service with keycloak (quay.io/keycloak/keycloak:26.7.4; start-dev --import-realm --db=dev-mem; hostname https://auth.mahfel.localhost; forwarded headers; health enabled), mount the realm import, rename the loopback port variable, and drop the dex-data volume.
2. deploy/keycloak/: add mahfel-realm.json (realm, realm roles, confidential API client with the reserved callback, public web client, six permanent dev identities with password 'password' and stable subject IDs) and a README; remove deploy/dex/.
3. Caddy: proxy auth.mahfel.localhost to keycloak:8080.
4. Config: point default.toml and local.toml.example at the Keycloak issuer and align scopes (openid/profile/email/roles); update .env.example and drop the stale OIDC issuer line.
5. mahfel-auth: replace the Dex discovery test with a Keycloak one and rename the claim-normalization test.
6. Docs: update overview, components, networking, local, getting-started, testing, repository-structure; add ADR 0018 and its nav entries.
7. Validate: docker compose config, Keycloak discovery via Caddy, mahfel-auth tests, just docs strict.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Realm is named 'rastai' (renamed per review), so AC #1/#2 wording reads 'realms/mahfel'; the implemented issuer is https://auth.mahfel.localhost/realms/rastai.

Implementation: Compose runs quay.io/keycloak/keycloak:26.7.4 (start-dev, --import-realm, --db=dev-mem, hostname https://auth.mahfel.localhost, forwarded headers, health enabled) with a /dev/tcp healthcheck; the dex service, its named volume, and deploy/dex were removed. deploy/keycloak/rastai-realm.json defines realm roles admin/teacher/student/moderator, a confidential mahfel-api client with the reserved callback, a public mahfel-web client, six permanent development identities (password 'password', stable subject IDs) and a mahfel-admin service-account client. Per review, mahfel-api manages Keycloak through the client-credentials grant (realm-management: manage-users/view-users/query-users/view-realm); no interactive administrator credentials are configured, so the admin console is not bootstrapped by default. Added [keycloak] backend settings (base_url, realm, admin_client_id, admin_client_secret), moved [oidc] to /realms/rastai with openid/profile/email/roles scopes, and dropped the stale MAHFEL_OIDC_ISSUER_URL line from .env.example. Replaced the Dex discovery test with a Keycloak testcontainers test that imports the realm, verifies discovery, and proves the service account can read realm users. Added ADR 0018 and updated the architecture, deployment, and development docs.

Validation: docker compose config ok; realm import verified against a live container (users, roles, clients, health); discovery correct directly and through Caddy (issuer https://auth.mahfel.localhost/realms/rastai); client-credentials token and Admin API user list succeed through Caddy; keycloak healthcheck healthy; MAHFEL_TESTCONTAINERS=1 keycloak test passed (~71s); mahfel-config and mahfel-auth tests pass; workspace clippy clean; just docs strict. The pre-existing dex container was removed as an orphan; the old dex-data volume still exists and can be pruned manually.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Replaced Dex with Keycloak as the single local and production OIDC provider. Compose imports the committed rastai realm, Caddy fronts it at https://auth.mahfel.localhost/realms/rastai, and mahfel-api is configured to manage the realm through a client-credentials service account rather than an administrator account. ADR 0018 records the decision; M2-04/M2-05 now develop against the real compatibility target.
<!-- SECTION:FINAL_SUMMARY:END -->
