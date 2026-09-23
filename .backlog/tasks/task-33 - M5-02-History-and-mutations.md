---
id: TASK-33
title: M5-02 History and mutations
status: Done
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-23 01:16'
labels: []
milestone: m-5
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 41000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Large histories need stable paging and controlled edits/deletes.

Implementation scope: Cursor pagination, edit window/policy, soft delete, revision/audit data and API contracts.

Dependencies: M5-01
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Pagination has no gaps/duplicates under inserts; mutation permissions tested.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add domain/application query and mutation contracts for room chat history, edits, deletes, and revision metadata. 2. Add a PostgreSQL migration for edits/revision audit data and indexes while keeping chat in PostgreSQL. 3. Implement stable keyset pagination ordered by (created_at, id), scoped to an authorized current room member. 4. Add atomic edit and soft-delete operations with explicit authorization: assume authors may edit/delete within 15 minutes and room managers may delete any message; retain revision history. 5. Expose history, edit, and delete REST contracts; regenerate OpenAPI/frontend types. 6. Verify cursor behavior under inserts and mutation permissions using disposable PostgreSQL integration tests, then run focused backend gates.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Policy confirmed by user: authors may edit or soft-delete their messages for 15 minutes; room managers may soft-delete any message. Implemented stable keyset history pagination, current-membership authorization, atomic edits and soft deletes, revision audit records, REST routes, OpenAPI contract, and generated frontend types. Validation passed: backend lint (workspace Clippy and SQL lint), backend formatting, disposable PostgreSQL migration/integration test, application tests (101 passed), API chat/OpenAPI focused tests (8 passed), API cargo check, OpenAPI export, and frontend typecheck. The full API bin suite had 104 passing and 24 failures because sandbox permissions denied local socket binds in existing WebSocket tests; focused chat and OpenAPI tests passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented M5-02 history pagination and message mutations with 15-minute author edit/delete permissions, manager soft-delete moderation, revision auditing, and regenerated API contracts. Verified with backend lint/format, database integration, focused API/application tests, OpenAPI generation, and frontend typecheck.
<!-- SECTION:FINAL_SUMMARY:END -->
