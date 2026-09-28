---
id: TASK-157
title: Replace sidebar browsers with navigation menus
status: Done
assignee: []
created_date: '2026-09-28 01:42'
updated_date: '2026-09-28 01:52'
labels: []
dependencies: []
ordinal: 165000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Use anchored room and session menus and move authorized creation into each menu; remove the separate creation rail action on desktop and mobile.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Room and session icons open anchored menus with API destinations
- [x] #2 Creation actions follow global room and selected-room session authorization
- [x] #3 Separate plus action is removed and focused checks pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Reuse shadcn dropdown menus for desktop and mobile room/session lists. 2. Gate session creation with the selected room membership API and preserve authenticated room creation. 3. Remove the separate plus action, update focused component coverage, and validate shell and Storybook.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented anchored nonmodal menus, room selector submenu, list pagination, API-based session creation permission, and integrated creation forms. Removed separate desktop and mobile plus actions.

Validation: shell suite 15 tests passed before one additional creation integration case; final menu suite 8 tests passed. Focused ESLint, TypeScript, formatting, diff check and Storybook build passed. No E2E tests written or run. Browser visual verification and deployment not performed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Replaced room/session browser dialogs with RTL-aware anchored shadcn menus, preserved paginated destinations and creation forms, gated session creation using the membership API, and removed the separate plus entry from desktop/mobile navigation.
<!-- SECTION:FINAL_SUMMARY:END -->
