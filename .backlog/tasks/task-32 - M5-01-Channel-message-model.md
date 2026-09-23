---
id: TASK-32
title: M5-01 Channel/message model
status: Done
assignee: []
created_date: '2026-09-08 00:31'
updated_date: '2026-09-23 00:25'
labels: []
milestone: m-5
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 40000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Classroom chat must outlive live sessions.

Implementation scope: Classroom/meeting channels, message IDs/content/author/timestamps, tombstones, indexes and repository.

Dependencies: M1, M2
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Messages persist and are scoped to authorized channel.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Model one persistent chat channel per generic Room plus durable messages with actor authorship, timestamps, and tombstone-ready state, following ADR 0020. A room channel persists across its Sessions. 2. Add a PostgreSQL migration with channel/message constraints and channel timeline indexes. 3. Add a dyn-compatible application store port and Postgres adapter. 4. Add an application command that atomically persists a message only when its author is a current room member; expose the narrow create-message API needed to exercise the acceptance criterion. 5. Review docs/contracts and validate the affected backend surface.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented generic room-scoped persistent chat channels and messages, tombstone-ready schema, timeline index, atomic current-member authorization, and authenticated create-message API with generated OpenAPI/frontend types. Validation: just backend fmt, just backend lint (Clippy and SQLFluff across migrations), just backend test-database (disposable PostgreSQL migration and persistence/authorization test), OpenAPI export, and frontend typecheck passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Delivered persistent room chat with PostgreSQL storage and member-authorized message creation. Verified formatting, backend lint, disposable PostgreSQL persistence/authorization integration test, OpenAPI generation, and frontend typecheck.
<!-- SECTION:FINAL_SUMMARY:END -->
