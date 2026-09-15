---
id: TASK-12
title: M2-04 OIDC provider abstraction
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-2
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 20000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Keycloak and future issuers must not leak vendor logic into handlers.

Implementation scope: Discovery, PKCE/state/nonce, callback validation, issuer config, claim normalization, clock/JWK refresh behavior.

Dependencies: M2-01, M2-03
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Fake issuer integration covers success, bad nonce/state/signature, and key rotation.
<!-- AC:END -->
