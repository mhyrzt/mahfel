---
id: TASK-13
title: M2-05 OIDC provider compatibility
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-19 19:31'
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
- [ ] #1 Provider-specific settings are externalized from application domain code
- [ ] #2 TinyAuth and Keycloak compatibility results are documented
<!-- DOD:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The rastai realm fixture defines the confidential mahfel-web client with the NextAuth redirect and post-logout URIs
- [ ] #2 A browser login against the Compose Keycloak succeeds and the API authenticates the resulting access token
- [ ] #3 Expected claims map deterministically into the application profile and roles
- [ ] #4 Logout ends both the Mahfel session and the Keycloak SSO session
- [ ] #5 The Keycloak README and docs no longer reference TinyAuth
<!-- AC:END -->
