---
id: TASK-14
title: M2-06 Identity linking
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
labels: []
milestone: m-2
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 22000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Existing users need safe account association across providers.

Implementation scope: Explicit linking ceremony, verified-email policy, collision handling, unlink safeguards and audit events.

Dependencies: M2-03, M2-04
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Same user owns multiple identities; implicit unverified-email linking is impossible; tests cover conflict.
<!-- AC:END -->
