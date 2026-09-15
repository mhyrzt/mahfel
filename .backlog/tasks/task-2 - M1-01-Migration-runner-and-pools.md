---
id: TASK-2
title: M1-01 Migration runner and pools
status: Done
assignee:
  - '@codex'
created_date: '2026-09-08 00:30'
updated_date: '2026-09-15 02:52'
labels: []
milestone: m-1
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 10000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Durable features need controlled schema lifecycle and bounded connections.

Implementation scope: SQLx migration command/startup policy, pool configuration, timeouts, health/metrics, disposable DB test harness.

Dependencies: M0
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Forward migration runs once safely; pool failure is observable; integration test provisions schema.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add typed database pool and migration-policy settings with validation and local overrides. 2. Extend mahfel-database with bounded pool construction, embedded SQLx migrations, explicit migration execution, and low-cardinality pool/probe metrics. 3. Add mahfel-api migrate mode plus optional migrate-on-start policy while keeping readiness tied to PostgreSQL. 4. Add a disposable PostgreSQL migration integration harness and focused just command. 5. Document operations and run package, workspace, container, and live database validation before a separate commit.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented typed bounded SQLx pool settings, embedded forward migrations, explicit API migrate mode, opt-in local migrate-on-start, PostgreSQL readiness probing, and low-cardinality pool/probe metrics. Added a disposable-database integration test and just test-database workflow. Validation: focused check/clippy/tests passed; live migration was idempotent across two runs; schema and one migration ledger entry verified; readiness changed 200 to 503 while PostgreSQL was unavailable while liveness stayed 200, then recovered; docker validation target passed exact fmt/check/clippy/test workspace gates; production API image and migration command passed; MkDocs strict and docker compose config passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added the first durable database lifecycle: validated bounded pools, embedded SQLx migrations, explicit and local startup migration policies, readiness/metrics, and a disposable PostgreSQL integration harness. Verified locally, in Docker, and against a live disposable database.
<!-- SECTION:FINAL_SUMMARY:END -->
