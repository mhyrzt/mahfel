---
id: TASK-38
title: M6-01 Object metadata and ACL
status: Done
assignee:
  - '@codex'
created_date: '2026-09-08 00:31'
updated_date: '2026-09-23 12:34'
labels: []
milestone: m-6
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 46000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Artifact bytes need authoritative ownership/permission metadata.

Implementation scope: Object key strategy, owner/classroom/session scope, status, size/MIME/checksum, ACL service and migrations.

Dependencies: M1, M2-07
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Metadata constraints/ACL matrix tests pass; keys are collision-resistant and non-guessable.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Define collision-resistant object keys and constrained PostgreSQL metadata; enforce context-aware ACL at the application boundary; expose a store port for upload state and listing.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Metadata and ACL checks passed: domain 86 tests, application 110 tests, PostgreSQL migration integration test.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added constrained PostgreSQL file metadata, random object keys, and context ACL rules. Domain, ACL, and disposable PostgreSQL tests pass.
<!-- SECTION:FINAL_SUMMARY:END -->
