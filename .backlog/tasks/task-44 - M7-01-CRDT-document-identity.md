---
id: TASK-44
title: M7-01 CRDT document identity
status: In Progress
assignee:
  - '@codex'
created_date: '2026-09-08 00:31'
updated_date: '2026-09-24 06:20'
labels: []
milestone: m-7
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 52000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Notes/boards need ownership, type, permission, and version boundaries.

Implementation scope: Document metadata/schema, classroom/session association, ACL, lifecycle and protocol version.

Dependencies: M1, M2-07, ADR 0012
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Unauthorized sync rejected; each document type/version resolves deterministically.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Define session-owned versioned notes and board documents, manager-controlled per-participant edit grants, durable metadata and ACL checks. Implement the remaining M7 tasks in dependency order with compact modules.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implementation present. Validation: backend Clippy and formatting, PostgreSQL migration integration, API unit suite, frontend typecheck, lint, formatting and 19 tests, production webpack build. Acceptance needs live multi-client/restart verification before marking done.
<!-- SECTION:NOTES:END -->
