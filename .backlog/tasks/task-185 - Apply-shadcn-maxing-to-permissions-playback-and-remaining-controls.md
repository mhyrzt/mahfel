---
id: TASK-185
title: Apply shadcn maxing to permissions playback and remaining controls
status: Done
assignee:
  - '@codex'
created_date: '2026-10-05 10:26'
updated_date: '2026-10-05 13:47'
labels:
  - frontend
  - ui
dependencies: []
documentation:
  - doc-1
ordinal: 193000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Complete approved doc-1 control migration while preserving permission locks, replay behavior, upload plumbing, feature layout, and intentional calendar/editor/media exceptions.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Permission locks and recording control behavior remain correct with shadcn primitives
- [x] #2 Eligible visible native controls are migrated and intentional exceptions documented
- [x] #3 Relevant frontend gates pass and documentation reflects the implementation
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Apply approved doc-1 slices 3 and 4, then consolidation: migrate checkboxes/table/slider/progress, eligible buttons, disclosure, tooltips, and feedback; preserve feature behavior, document exceptions, validate.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Full Vitest suite passed: 65 files / 197 tests. Final focused selection/preferences tests passed: 2 files / 5 tests. Formatting, lint (zero warnings), strict typecheck, Storybook build, and whitespace checks passed. Browser unavailable; no visual/runtime proof. Standard production build is blocked by Turbopack worker port permission, including escalated retry; Webpack fallback failed parsing TypeScript --showConfig. Available Bun 1.3.14 differs from the unchanged 1.4.2 pin. Details and source census are in doc-1. Implementation is applied; leave task In Progress and acceptance criterion 3 unchecked until production packaging is verified.

Migrated permission Table/Checkbox, replay Slider/Checkbox, export Progress, disclosure, button tiles, composer, notes tooltips, shared alerts/empty/spinner and breadcrumb. Final census is zero eligible raw controls and four documented native file inputs.

Release follow-up: format/lint/types and all 197 tests pass again. CI-equivalent production Docker build with pinned Bun 1.4.2 and frozen lockfile passed; the resulting image /api/health returns ok. Production packaging gap resolved. Browser proof remains unavailable and is explicitly documented.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Migrated permissions/playback and eligible product controls to shared shadcn primitives, with enforced exceptions and documentation. All local frontend gates and pinned-Bun Docker production build/health pass; browser visual and live interaction verification remain unclaimed.
<!-- SECTION:FINAL_SUMMARY:END -->
