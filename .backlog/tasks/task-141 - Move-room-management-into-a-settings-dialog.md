---
id: TASK-141
title: Move room management into a settings dialog
status: Done
assignee: []
created_date: '2026-09-25 17:37'
updated_date: '2026-09-25 17:42'
labels:
  - frontend
dependencies: []
ordinal: 149000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Keep the room detail focused on sessions and files. Managers open a tabbed dialog for invite links, admission queue, and joining policy. Remove the room conversation for now and simplify session action buttons.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Managers can open a room settings dialog with invite links, admission queue, and admission policy tabs
- [x] #2 Members do not see manager settings and manager controls are absent from the page body
- [x] #3 Room conversation is absent from the detail page
- [x] #4 Session actions use icon buttons with accessible Join session and Playback tooltips
- [x] #5 Component tests and Storybook cover the updated room page and dialog
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Compose manager settings in a client dialog using existing Dialog and Tabs. 2. Update room page and session action controls. 3. Add component tests and stories, run frontend gates.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Room settings uses existing Dialog and Tabs; admission requests load only while their tab is active. Validation: frontend fmt, lint, typecheck, 76 Vitest tests, and Storybook build passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Removed room conversation, moved manager controls to a three-tab settings dialog, and changed session actions to accessible icon buttons. Frontend checks and Storybook build passed.
<!-- SECTION:FINAL_SUMMARY:END -->
