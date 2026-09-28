---
id: TASK-155
title: Simplify account menu and preferences
status: Done
assignee: []
created_date: '2026-09-27 14:10'
updated_date: '2026-09-27 14:50'
labels:
  - frontend
  - design
dependencies: []
ordinal: 163000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Keep the avatar menu focused on account navigation and sign-out, and present appearance, calendar, and language choices clearly in account settings across English and Persian.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Avatar menu contains account identity, account settings, and sign-out without quick preference controls
- [x] #2 Account settings offers keyboard-accessible appearance, calendar, and language controls with clear selected states
- [x] #3 Landing preference controls remain compact and usable on narrow screens
- [x] #4 Storybook and focused component tests reflect the updated surfaces
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Replace the custom avatar popup with the existing dropdown primitive; redesign settings preferences as clear rows with segmented choices; reduce landing header controls to compact menus; validate tests, Storybook, and responsive behavior.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Replaced hover quick-settings popup with a focused Radix account menu. Moved theme, calendar, and language choices into segmented rows in account settings, and made landing controls compact dropdowns. Verified English and Persian Storybook views, mobile widths without horizontal overflow, keyboard-accessible buttons and menu, focused tests, lint, typecheck, formatting, Storybook build, and Next production build. Full suite had one stale account trigger-name assertion; updated it and the affected tests passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Simplified the avatar menu and redesigned account preferences and landing controls; validated focused interactions, responsive Storybook views, frontend checks, and production packaging.
<!-- SECTION:FINAL_SUMMARY:END -->
