---
id: TASK-9
title: M2-01 Identity schema
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-2
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 17000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: One Mahfel user may have multiple login identities.

Implementation scope: Users, identities (issuer, subject), verified emails, avatar source, local credential table, uniqueness/migrations.

Dependencies: M1
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Constraint tests prevent identity takeover and allow multiple identities per user.
<!-- AC:END -->
