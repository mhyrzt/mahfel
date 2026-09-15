---
id: TASK-10
title: M2-02 Local password lifecycle
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-2
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 18000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Local users need secure credential creation and verification.

Implementation scope: Argon2id parameters, OS randomness/salts, constant-time verification through maintained crate, password change/reset token boundaries.

Dependencies: M2-01
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Known-vector/rehash tests pass; plaintext/password material is never logged or stored.
<!-- AC:END -->
