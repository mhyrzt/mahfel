---
id: TASK-153
title: Polish room chat and live message notices
status: In Progress
assignee: []
created_date: '2026-09-27 09:44'
updated_date: '2026-09-27 10:27'
labels:
  - frontend
  - bug
dependencies: []
ordinal: 161000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Make the existing room chat available in the real room page, style grouped messages with the local shadcn Bubble and Message primitives, and notify session participants when a new room message arrives.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Room detail displays chat with grouped sender-aligned bubbles, timestamps, and a compact composer
- [ ] #2 New messages received during a live session show a dismissible notification without replaying history
- [ ] #3 English, Persian, and Storybook states reflect the real chat layout
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Mounted room chat in the real room detail page, grouped and aligned messages with local Bubble and Message primitives, added timestamps and near-bottom auto-scroll, simplified composer, and added dismissible live session notices for new room messages. Session client notification test and full frontend suite passed. Live WebSocket delivery remains unverified.
<!-- SECTION:NOTES:END -->
