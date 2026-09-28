---
id: TASK-145
title: Use Storybook workspace design in the live application
status: Done
assignee:
  - '@codex'
created_date: '2026-09-26 11:23'
updated_date: '2026-09-26 11:45'
labels: []
dependencies: []
ordinal: 153000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The Workspace Home story is currently disconnected from the /workspace route, which redirects to a room when rooms exist. Connect the designed workspace surface to real room and session data without displaying fabricated fixture values.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The authenticated /workspace route renders the Workspace Home layout for users with rooms
- [x] #2 Room and session links open their existing live routes and reflect real API state
- [x] #3 Empty and unavailable states are truthful and usable at desktop and mobile sizes
- [x] #4 The calendar reflects the current month and actual scheduled dates instead of fixed Storybook fixture dates
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Replace /workspace first-room redirect with the WorkspaceHome composition. 2. Adapt actual room and scheduled-session API responses to its view models without fabricated fields, with bounded reads. 3. Make the calendar current and make session actions navigate to real routes. 4. Check responsive Storybook and live behavior with focused frontend gates.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Frontend lint, typecheck, formatting, Storybook build, and Docker production build passed. The local authenticated Compose probe loaded /workspace with overview and calendar markup and opened a seeded room. Mobile layout and the session navigation action were checked in source and Storybook build, not through a live browser interaction.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Rendered the Storybook workspace layout on /workspace with real room and session data, current interactive calendar dates, usable empty and schedule-unavailable states, and session links to live routes.
<!-- SECTION:FINAL_SUMMARY:END -->
