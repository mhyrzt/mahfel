---
id: TASK-156
title: Refine presentation carousel and sidebar conversations
status: In Progress
assignee:
  - '@codex'
created_date: '2026-09-28 01:10'
updated_date: '2026-09-28 09:28'
labels:
  - frontend
  - bug
dependencies: []
ordinal: 164000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Use the shadcn Embla carousel for synchronized presentation navigation, move room conversation into a reusable sidebar entry, align sidebar surfaces, and remove redundant creation captions from room session cards. Session chat scope is awaiting the user choice because the current backend exposes one room channel.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Presentation supports presenter navigation and follows remote page changes without viewer controls
- [x] #2 Room conversation opens from a sidebar icon and is absent from page content
- [x] #3 Message and Bubble styling fits the sidebar in English and Persian
- [x] #4 Sidebar surfaces and session cards are visually consistent and Storybook uses the real components
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add the Embla carousel and a shared presentation view. 2. Move conversation into a reusable sidebar and wire session message notices to open it. 3. Refine backgrounds and session captions. 4. Validate focused component behavior, lint, types, and Storybook.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Added Embla carousel with presenter-only swipe/keyboard controls and remote page synchronization, extracted the real PresentationViewer for Storybook, moved conversation into a reusable shadcn sidebar sheet, wired live message notices to open it, removed redundant Created captions, and aligned rail/panel backgrounds. Focused tests: 24 passed across 5 files; TypeScript, focused ESLint, Prettier, diff checks, and Storybook build passed. Browser tool is unavailable, so runtime visual proof is not claimed. Session panel currently uses the room conversation; isolated session chat remains pending the scope answer and no backend contract was changed.

Extracted ConversationPanel so Storybook room and session previews use the same composer and message layout as the app. Final focused suite: 24 passed; focused lint, types, formatting, diff check, and Storybook packaging passed. Production packaging remains unverified: Turbopack fails due sandbox port binding, and webpack fallback fails parsing TypeScript --showConfig. Task remains in progress while the separate-versus-shared session chat decision is pending.
<!-- SECTION:NOTES:END -->

## Comments

<!-- COMMENTS:BEGIN -->
author: @deepseek
created: 2026-09-28 09:28
---
Session chat scope decision made: separate session-scoped messages were implemented under TASK-158 (optional session_id on chat messages, migration 0026, session_id filter on /v1/rooms/{room_id}/messages, session_id in chat_event). The session sidebar now renders a session-scoped RoomChat instead of the shared room conversation, so the pending decision in this task is resolved.
---
<!-- COMMENTS:END -->
