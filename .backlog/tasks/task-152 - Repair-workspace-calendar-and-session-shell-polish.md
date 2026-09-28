---
id: TASK-152
title: Repair workspace calendar and session shell polish
status: In Progress
assignee: []
created_date: '2026-09-27 04:41'
updated_date: '2026-09-27 10:27'
labels:
  - frontend
  - bug
dependencies: []
ordinal: 160000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Apply the saved Persian calendar preference to the workspace, restore centered room layout and greeting time, add a rail theme toggle, improve session sidebar hover behavior and motion, and keep live sessions within the viewport.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Persian calendar uses Shamsi months and Saturday-first compact Persian weekdays
- [ ] #2 Workspace rooms are centered with a localized greeting and current time
- [ ] #3 Sidebar theme button toggles light and dark above the avatar
- [ ] #4 Session sidebar buttons have stable hover labels and live stage does not scroll the page
- [ ] #5 Session join and surface transitions respect reduced motion and the end action is integrated into the toolbar
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented Persian month navigation and compact weekday labels, centered workspace greeting/time, sidebar theme toggle and tooltips, reduced-motion-aware transitions, integrated end action, and viewport-fit session shell. Frontend lint/typecheck and 105 Vitest unit/component tests passed. User-reported screenshot predates these changes; live visual verification remains pending.

Added a seconds clock beneath the greeting, breadcrumb, large group stage sizing, and full screen Storybook session layout. Static Storybook review showed English and Persian 14 participant stages with document height equal to viewport height. Full frontend suite passed 107 tests before final layout-only edits; focused stage tests, lint, typecheck, and Storybook build passed afterward.
<!-- SECTION:NOTES:END -->
