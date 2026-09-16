---
id: TASK-13
title: M2-05 OIDC provider compatibility
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-16 22:10'
labels: []
milestone: m-2
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 21000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Local development needs a lightweight OIDC provider while production must remain compatible with the organization’s Keycloak deployment.

Implementation scope: Verify TinyAuth discovery and authorization-code login using the local Compose fixture; verify Keycloak realm/client settings, redirect/logout flows, avatar and role claim mapping, and configuration-only issuer switching. Document identity migration/linking and provider-specific differences.

Dependencies: M2-04
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 TinyAuth local OIDC login works against the shared Compose fixture
- [ ] #2 Real Keycloak login/logout works and expected claims map deterministically
<!-- AC:END -->





## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Provider-specific settings are externalized from application domain code
- [ ] #2 TinyAuth and Keycloak compatibility results are documented
<!-- DOD:END -->
