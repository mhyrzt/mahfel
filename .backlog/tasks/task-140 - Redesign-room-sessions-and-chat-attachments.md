---
id: TASK-140
title: Redesign room sessions and chat attachments
status: Done
assignee:
  - '@codex'
created_date: '2026-09-25 17:03'
updated_date: '2026-09-25 17:19'
labels:
  - frontend
dependencies: []
references:
  - 'https://ui.shadcn.com/docs/components/base/attachment'
  - 'https://ui.shadcn.com/docs/components/base/message'
ordinal: 148000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Make room sessions the primary content with schedule, live state, expandable attached and presentation-used files, and clearly disabled playback. Use shadcn Attachment and Message composition for file and chat surfaces.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Room session rows show schedule, status, files, navigation, and disabled playback in English and Persian
- [x] #2 Session file disclosure separates currently presented files from other session attachments
- [x] #3 Room and message files use attachment cards; chat rows use message composition and retain actions
- [x] #4 Storybook and unit tests cover full room and component states
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Make session cards the visual focus and show schedule, state, navigation, disabled playback. 2. Load session files on disclosure, separating current presentation assets from other attachments. 3. Compose shared Attachment cards and Message/Bubble rows for files and chat; preserve upload, preview, download, moderation, and reply behavior. 4. Add tests and Storybook states; run frontend gates.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Session disclosures fetch ready session files and current presentation file IDs; selected room files appear under Used in presentation. Chat attachments are uploaded to the existing chat_message file context after the message exists. Frontend fmt, lint, typecheck, 72 unit tests, Storybook build, and docs build pass. Next production build is blocked by sandbox port binding (Turbopack); webpack fallback cannot parse TypeScript --showConfig output here.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Redesigned the room page around scheduled/live session rows and expandable file groups; adopted Attachment and Message compositions for room files and chat while retaining actions. Verified with frontend gates, 72 tests, Storybook, and docs build.
<!-- SECTION:FINAL_SUMMARY:END -->
