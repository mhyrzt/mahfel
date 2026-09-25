---
id: TASK-142
title: Complete responsive room management and session files
status: Done
assignee: []
created_date: '2026-09-25 18:02'
updated_date: '2026-09-25 19:10'
labels:
  - frontend
  - backend
dependencies: []
ordinal: 150000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Refine the room detail UI for mobile and populated rooms, provide manager member controls, and make session file selection usable from computer or room bank.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Room detail has no standalone file sidebar and header controls are icon buttons with accessible labels
- [x] #2 Settings groups admission policy with requests and lets managers list, add, remove, and change member roles
- [x] #3 Session files offer a responsive upload/drop target and room-bank selection without Storybook API errors
- [x] #4 Storybook includes a populated room with more than ten sessions and mobile presentation
- [x] #5 Frontend and backend tests cover new behavior and relevant checks pass
- [x] #6 Room and Room activity eyebrow labels are absent; sessions sort oldest to newest
- [x] #7 Session attachments start expanded in a bounded scrollable grid
- [x] #8 Authorized room managers can open each session card settings dialog and update title or schedule
- [x] #9 Room detail keeps the room title while omitting the session count, visibility, and admission summary
- [x] #10 Session list has no visible Sessions heading or separate Join session control; session title opens the session and disabled playback uses a film icon
<!-- AC:END -->



## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add manager-only member listing and durable room-file/session links in the API, then regenerate REST contract. 2. Refine room settings and member management. 3. Build expanded scrollable session attachment grid, upload/drop and room-bank picker, plus per-session settings. 4. Add populated mobile stories and tests, run component gates, split commits.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Added manager-only member listing and durable same-room file/session links with migration 0024; member add uses user UUID because no user search API exists. Storybook file fixtures avoid live API calls and include a 13-session mobile view. Backend fmt/lint/test including PostgreSQL Testcontainers, frontend fmt/lint/typecheck/83 tests, Storybook build, and docs passed. Next production build remains unverified because Turbopack fails binding a port while processing Excalidraw CSS even with sandbox escalation.

Removed the visible Sessions heading and redundant join icon; session titles remain links and playback uses the Film icon.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Refined room layout and mobile stories, added tabbed member management and admission toggle, persistent session file selection, expanded scrollable attachment grids, and per-session settings. Component and backend checks passed; production Next build is blocked by environment port binding.

Session cards now use title links for entry, with Film for the disabled playback action.
<!-- SECTION:FINAL_SUMMARY:END -->
