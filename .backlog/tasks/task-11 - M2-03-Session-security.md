---
id: TASK-11
title: M2-03 Session security
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-2
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 19000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Browser authentication needs revocation, expiry, and CSRF-safe transport.

Implementation scope: Decide opaque session versus signed token, secure cookie flags, rotation/revocation, CSRF approach, session store and middleware.

Dependencies: M2-02
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Login/logout/expiry/rotation tests pass and stolen old session is rejected after rotation.
<!-- AC:END -->
