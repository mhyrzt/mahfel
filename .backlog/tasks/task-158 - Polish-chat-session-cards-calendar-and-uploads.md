---
id: TASK-158
title: 'Polish chat, session cards, calendar, and uploads'
status: Done
assignee:
  - '@deepseek'
created_date: '2026-09-28 08:51'
updated_date: '2026-09-28 09:59'
labels:
  - frontend
  - backend
  - bug
dependencies: []
references:
  - TASK-153
  - TASK-156
  - TASK-152
  - TASK-149
priority: medium
ordinal: 166000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Address a batch of user-reported defects across the room and session experience: live participant counts on ongoing session cards, a cleaner workspace calendar, chat interaction and identity polish with composer-time attachments and session-scoped conversations, working session file deletion, and video upload support.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Ongoing session cards on the room page show the number of participants currently in the session
- [x] #2 Room chat messages expose a single overflow menu with edit and delete actions instead of inline buttons
- [x] #3 Chat avatars derive from the message author consistently: the signed-in user sees a self label and other authors show a stable identity, with the avatar adjacent to the message and the author name inside the message content
- [x] #4 Chat attachments are chosen while composing and uploaded on send; already-sent messages no longer offer attaching files
- [x] #5 A session conversation fetches only messages scoped to that session, while the room conversation fetches only messages that are not scoped to any session
- [x] #6 The chat backend stores an optional session scope on messages with a forward migration, exposes it in the message APIs, and carries it in realtime chat events
- [x] #7 Deleting a session file succeeds for authorized owners and managers and surfaces a delete-specific error message when it fails
- [x] #8 MP4 and WebM video files upload and verify within the existing 50 MB file limit
- [x] #9 English and Persian copy, Storybook states, and focused tests cover the changed surfaces, and the relevant frontend and backend checks pass
- [x] #10 The workspace month calendar uses a clearer day grid, weekday header, selected and today states, and schedule markers
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Backend: add a nullable session scope to chat messages (migration 0026), thread it through domain, application, postgres, REST DTOs, and realtime chat events; add a session scoping filter to message history. 2. Frontend: regenerate contracts and route room vs session conversation fetches by scope. 3. Chat UX: overflow menu for edit/delete, consistent author identity and layout, composer-time attachments, and remove post-send attach. 4. Session cards: fetch live roster counts for ongoing sessions. 5. Calendar: improve the workspace month calendar. 6. Files: fix delete error handling and allow MP4/WebM with signature verification. 7. Validate with focused tests, lint, typecheck, Storybook, and backend checks.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
2026-09-28: Backend chat now stores an optional session scope (migration 0026) validated against the message's room; replies inherit their root scope. Realtime chat_event frames carry session_id, and GET/POST /v1/rooms/{room_id}/messages accept a session_id filter. File deletion now detaches a file from live presentation selections (clearing an emptied selection) before the deleting transition and returns a conflict only when the file is embedded in a collaboration document; the frontend surfaces a delete-specific message. File validation accepts video/mp4 (ftyp box) and video/webm (EBML header) within the existing 50 MiB limit. Date picker refactor was limited to the workspace month calendar; the chat composer/lexical change was intentionally skipped per the user's choice.

Validation: cargo fmt --check, cargo clippy --workspace --all-targets --all-features -D warnings, backend unit tests (88 domain/application + 149 api bins), PostgreSQL Testcontainers migration test with new session-scope and presented-file detach regressions, sqlfluff lint migrations. Frontend: bun run format, lint, typecheck, 119 Vitest tests, Storybook build, and next production build all pass. Authored session scope is consistent with existing session observability, which is gated on room membership; per-session WebSocket authorization was not added because chat subscriptions remain room-level.

2026-09-28 follow-up from review: message history and reply pages now return each message's ready attachments (new FileStore::list_for_messages), and the chat list renders them inline as a Telegram-style media album plus file cards instead of an Attachments disclosure. Message chrome moved inside the bubble: author label and the three-dot overflow menu share a header row, with the timestamp inside the bubble, and bubbles use a uniform width. The conversation sidebar sheet hides its built-in close button. Video tiles capture a first-frame poster client-side. Storybook gains action, inline-album, video, edited, and session-scope states for the message list.

Follow-up validation: frontend format/lint/typecheck, 125 Vitest tests, Storybook build, next production build; backend fmt/clippy, mahfel-api bin tests, and the PostgreSQL Testcontainers migration test with a list_for_messages assertion.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added session-scoped room chat (migration 0026), a chat overflow menu with edit/delete, consistent in-bubble author chrome, composer-time attachments with inline media albums, live participant counts on ongoing session cards, a clearer workspace month calendar, working session-file deletion via presentation detach, MP4/WebM uploads, and a close-button-free conversation sidebar. Verified with backend fmt/clippy/unit tests, the PostgreSQL Testcontainers migration test, sqlfluff, and frontend format/lint/typecheck/125 tests/Storybook/Next build.
<!-- SECTION:FINAL_SUMMARY:END -->
