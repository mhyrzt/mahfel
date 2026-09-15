---
id: TASK-107
title: Split frontend and backend Just workflows
status: Done
assignee:
  - '@codex'
created_date: '2026-09-15 23:18'
updated_date: '2026-09-15 23:37'
labels: []
dependencies: []
ordinal: 115000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Frontend and backend contributors need focused local command surfaces while the repository root remains the unified entry point.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 backend/justfile exposes backend development, migration, format, lint, test, database-test, and build commands
- [x] #2 frontend/justfile exposes install, development, format or lint, typecheck, test, and build commands supported by package scripts
- [x] #3 Root justfile delegates shared recipes to the component justfiles while retaining compose and documentation commands
- [x] #4 Root and component recipe listings and representative delegated commands validate successfully
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add component-local backend and frontend justfiles that load the root environment and expose their native workflows. 2. Replace root shell-directory duplication with explicit delegation while keeping existing root recipe names stable. 3. Add generic backend/frontend proxy recipes for component-specific commands. 4. Document and validate recipe discovery plus representative delegated checks without changing application behavior.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Added component-local Justfiles, stable root delegation, generic backend/frontend proxy recipes, readable database-test shell logic, and frontend format commands. Root and component listings, proxy dry-runs, and root formatting passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Split frontend and backend workflows into local Justfiles while preserving the root command surface as a delegating entry point.
<!-- SECTION:FINAL_SUMMARY:END -->
