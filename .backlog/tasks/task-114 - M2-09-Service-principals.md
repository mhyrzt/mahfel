---
id: TASK-114
title: M2-09 Service principals
status: To Do
assignee:
  - '@mahyar'
created_date: '2026-09-19 12:34'
labels: []
milestone: m-2
dependencies:
  - TASK-9
  - TASK-11
  - TASK-15
documentation:
  - docs/adr/index.md
priority: medium
ordinal: 122000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: external systems such as lms-core must create, populate, and manage rooms through the API, but the identity model is user-only (Principal is a UserId and rooms.created_by references users). Today a service cannot be an owner or manager without impersonating a human user.

Implementation scope: a first-class machine actor distinct from users, its authentication mechanism (issuer-issued client-credentials/service-account token or an equivalent documented mechanism), provisioning and revocation, and room ownership/management by a service principal. This is deferred from M1-04, which keeps rooms user-owned.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A service principal authenticates as a first-class actor distinct from any user row, and Mahfel never represents a service as an impersonated user
- [ ] #2 A service principal can create rooms and manage members and settings exactly like a user-owned room, and service-authored rooms record the service principal as owner
- [ ] #3 Service principals can be provisioned, rotated, and revoked; revocation rejects subsequent requests
- [ ] #4 Authorization matrix tests cover service principals next to users and stay deny-by-default
- [ ] #5 An ADR records the chosen service-authentication mechanism, including how local development provisions a service principal
<!-- AC:END -->
