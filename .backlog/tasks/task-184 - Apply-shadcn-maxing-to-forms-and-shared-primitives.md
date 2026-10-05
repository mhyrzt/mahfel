---
id: TASK-184
title: Apply shadcn maxing to forms and shared primitives
status: Done
assignee:
  - '@codex'
created_date: '2026-10-05 10:26'
updated_date: '2026-10-05 10:52'
labels:
  - frontend
  - ui
dependencies: []
documentation:
  - doc-1
ordinal: 192000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Apply approved doc-1 shared-primitives and forms/settings migration, retaining Mahfel theme tokens, locale cookies, RTL, API values and pending/error behavior.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Ordinary forms and settings use shadcn fields, switches, selects, and exclusive selection primitives while preserving values and pending/error behavior
- [x] #2 Theme and locale semantics remain intact and new controls have keyboard and accessible label support
- [x] #3 Relevant tests, formatting, lint, types, and Storybook checks pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Apply approved doc-1 slices 1 and 2: adapt current registry primitives to local helpers and individual Radix packages, preserve semantic themes, provide direction context, migrate form labels/errors and switches/selects/radios; validate focused behavior and frontend gates.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Adapted official registry primitives; added individual Radix packages and animation utilities; migrated forms/settings and preference radio semantics. Typecheck and 14 focused tests across 7 files pass. Full gates and browser review pending.

Full Vitest suite passed: 65 files / 197 tests. Final focused selection/preferences tests passed: 2 files / 5 tests. Formatting, lint (zero warnings), strict typecheck, Storybook build, and whitespace checks passed. Browser unavailable; no visual/runtime proof. Standard production build is blocked by Turbopack worker port permission, including escalated retry; Webpack fallback failed parsing TypeScript --showConfig. Available Bun 1.3.14 differs from the unchanged 1.4.2 pin. Details and source census are in doc-1.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added adapted shadcn/Radix foundation and migrated forms/settings. Verified controlled failure behavior, keyboard radio selection and accessibility names with tests; format/lint/types/Storybook pass. Browser and production-build limitations are documented in doc-1.
<!-- SECTION:FINAL_SUMMARY:END -->
