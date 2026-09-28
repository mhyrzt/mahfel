---
id: TASK-149
title: Refine live session workspace layout and files
status: In Progress
assignee: []
created_date: '2026-09-27 00:35'
updated_date: '2026-09-27 10:27'
labels:
  - frontend
  - bug
dependencies: []
ordinal: 157000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Fix RTL session spacing, move live files and presentation selection into the session sidebar, improve collaboration panel height and image previews, and relocate the live session end action.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Live session files and presentation selection live in the sidebar, with no duplicate footer file section
- [ ] #2 Notes, whiteboard, and presentation use the available viewport height
- [ ] #3 Image attachments show preview thumbnails and remain accessible
- [ ] #4 Live session panel opens beside the persistent room, session, and profile rail in RTL and LTR
- [ ] #5 Invite links are a session sidebar item and do not appear below the live page
- [ ] #6 End session for everyone is in the live toolbar and the top action card is removed
- [ ] #7 A manager can draw on the live whiteboard
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Correct the official sidebar width and rail offset, then place files and invitation links in its panels. 2. Move the live end action into the session toolbar. 3. Preserve full-height collaboration surfaces and manager whiteboard editing. 4. Verify component behavior, type safety, Storybook, and production packaging.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
2026-09-27: Corrected Tailwind sidebar width and fixed-panel offset, moved invite links into the rail panel and end-for-everyone into the toolbar, restored manager whiteboard editability, and added focused component tests. Source checks and Storybook/webpack production builds pass; authenticated browser visual proof is unavailable in this workspace.

Added one drag-and-drop file library in the live sidebar with present icons for PDF and images, removed the duplicate presentation picker, and improved presentation empty and loading states. Static Storybook inspection confirmed English and Persian live stages fit a 985x615 viewport without page overflow. Authenticated file transfer, presentation, and whiteboard actions still need live verification.
<!-- SECTION:NOTES:END -->
