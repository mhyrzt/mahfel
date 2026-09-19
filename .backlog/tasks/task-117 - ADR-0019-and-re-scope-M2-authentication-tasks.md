---
id: TASK-117
title: ADR 0019 and re-scope M2 authentication tasks
status: Done
assignee:
  - '@mahyar'
created_date: '2026-09-19 19:24'
updated_date: '2026-09-19 19:34'
labels:
  - architecture
  - auth
  - frontend
milestone: m-2
dependencies: []
priority: high
ordinal: 125000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Record and socialize the authentication architecture change: mahfel-web is the browser OIDC client (NextAuth + Keycloak) with a narrow BFF, and mahfel-api is a resource server that verifies Keycloak access tokens and provisions local users by (issuer, subject). This supersedes the API-owned authorization-code flow assumed by M2-03/04/05 and the blanket no-BFF rule.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 ADR 0019 is accepted and states the frontend OIDC + BFF + API resource-server model, the reason the BFF is allowed, and that it supersedes the API authorization-code flow
- [x] #2 backend/AGENTS.md and frontend/AGENTS.md plus docs/architecture describe the model and the BFF's narrow scope
- [x] #3 TASK-11, TASK-12, and TASK-13 are re-scoped to browser session security, API token verification and identity sync, and Keycloak login/logout respectively
- [x] #4 No document claims the API runs an authorization-code login or owns a browser session
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Write ADR 0019 (frontend OIDC + narrow BFF; API resource server; supersedes the API authorization-code flow) and add it to the ADR index and MkDocs nav.
2. Update backend/AGENTS.md and frontend/AGENTS.md: the BFF exception and its narrow scope, and the API resource-server responsibilities.
3. Update docs/architecture components/overview/data-flow/networking to describe the browser -> mahfel-web (NextAuth) -> BFF -> mahfel-api (token verification) flow and drop the API callback claim.
4. Re-scope TASK-11 (browser session security), TASK-12 (API token verification + identity sync), TASK-13 (Keycloak login/logout end-to-end).
5. Validate: just docs strict.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Added ADR 0019 (frontend OIDC + narrow BFF; mahfel-api as resource server) with index/nav entries, annotated ADR 0018 as partially superseded, and updated architecture docs (overview diagram + flow, components, networking, data-flow) plus backend/frontend AGENTS. Re-scoped TASK-11 (browser session security), TASK-12 (API token verification + identity provisioning), and TASK-13 (Keycloak login/logout end-to-end, TinyAuth removed). Created TASK-119 (seed Keycloak users + avatar_url) and TASK-121 (WebSocket token verification); archived the duplicate TASK-118/TASK-120. Fixed the Keycloak README's stale API-callback claim. Validation: just docs strict passes.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Recorded the authentication architecture change in ADR 0019: mahfel-web is the browser OIDC client with a narrow BFF, and mahfel-api is a resource server that verifies Keycloak tokens and provisions local users by (issuer, subject). Updated AGENTS and architecture docs, re-scoped the M2 auth tasks, and aligned the Keycloak fixture documentation. No document claims the API runs an authorization-code login.
<!-- SECTION:FINAL_SUMMARY:END -->
