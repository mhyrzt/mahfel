---
id: TASK-3
title: M1-02 User/room/session core
status: Done
assignee:
  - '@codex'
created_date: '2026-09-08 00:31'
updated_date: '2026-09-15 03:02'
labels: []
milestone: m-1
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 11000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Routes need durable identifiers and invariants without a complex schema.

Implementation scope: Minimal users, rooms, classroom sessions, timestamps/statuses, repositories and migrations.

Dependencies: M1-01
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Repository tests create/read each model and data survives process restart.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Define minimal typed user, room, and classroom-session domain models with explicit status and timestamp invariants. 2. Add forward SQL migrations with small relational constraints and indexes. 3. Add repository methods for create/read operations without coupling domain types to SQLx. 4. Exercise all models against a disposable PostgreSQL database, including pool close/reconnect persistence. 5. Document the schema boundary, run focused and workspace validation, then commit the task independently.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented typed UUID identifiers and explicit storage-stable statuses for user, room, and classroom session models. Added constrained PostgreSQL tables and indexes plus SQLx repository adapters that keep domain types independent of persistence. The disposable integration test creates all three records, closes the pool, reconnects, and reads each record back. Validation: domain/database check, strict clippy, and tests passed; live PostgreSQL integration test passed with 1 test; exact full workspace fmt/check/clippy/test passed in the production-compatible Docker validation target; MkDocs strict and Compose config passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added the minimal durable user, room, and classroom-session core with typed domain models, constrained migrations, repository adapters, and restart-persistence coverage against disposable PostgreSQL.
<!-- SECTION:FINAL_SUMMARY:END -->
