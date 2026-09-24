---
id: TASK-126
title: M4-10 Frontend session state and roster
status: Done
assignee:
  - '@mahyar'
created_date: '2026-09-20 18:58'
updated_date: '2026-09-24 03:11'
labels:
  - frontend
milestone: m-4
dependencies:
  - TASK-121
  - TASK-27
  - TASK-125
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 134000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Client-side session signaling through a short-lived single-use WebSocket ticket, join and leave flow, and a roster reconciled from stable participant identities. Waiting or lobby state is represented with media controls hidden until admission. Caller capabilities gate controls; authorization remains in mahfel-api.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The roster reflects join, leave, remove, and block events and survives reconnect without duplicate participants.
- [x] #2 Reconnect re-establishes signaling and reconciles the stable participant identity.
- [x] #3 Lobby policy shows a waiting state and no media controls before admission.
- [x] #4 Controls the caller cannot use are hidden or disabled from the capability payload.
- [x] #5 Expired or unauthorized tokens produce actionable UX instead of silent failure.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add ADR 0025 for short-lived, single-use tickets replacing browser-visible bearer auth. 2. Add Redis-backed ephemeral ticket storage, authenticated POST /v1/ws/tickets issuance, and ticket consumption as the first WebSocket auth frame while preserving native bearer auth. 3. Export OpenAPI and AsyncAPI contracts and regenerate frontend types. 4. Add the exact /v1/ws/tickets BFF allowlist entry and implement direct-to-API typed signaling with auth, welcome, bounded backoff reconnect, cleanup, and actionable auth states. 5. Mount /sessions/[sessionId], load the caller-scoped capability payload server-side, join/leave through the socket, reconcile snapshots and participant events, represent waiting/rejected/disconnected states, and allow a deliberate retry after admission review. The manager-only session details route is unavailable to ordinary participants, so use a generic localized session title; current WebSocket roster contract provides stable participant IDs only. 6. Run scoped API/Redis, frontend, formatting, diff, and docs checks; refresh specs and update documentation.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
2026-09-22: UI-first foundation authorized. Axios and openapi-typescript were added to frontend dependencies; API contract types will be generated from the backend-owned /v1/openapi.json document.

2026-09-22: Generated the initial REST type artifact from the code-first snapshot. The Axios feature adapter and session UI state remain the next implementation slice.

2026-09-22: Split Axios entry points into `lib/api/browser.ts` (same-origin BFF only) and `lib/api/server.ts` (server-only authenticated direct API client). Added typed API error mapping and contract re-exports; focused tests live under `lib/api/tests/`.

2026-09-22: Added the pure session reducer at `features/sessions/session-state.ts`. It models loading, waiting, live, ended, rejected, and unavailable states; normalizes reconnect roster snapshots by participant ID; handles participant upsert/removal; retains stable self identity; and disables media outside an authorized live state. Focused reducer tests pass.

2026-09-22: Composed `SessionScreen` from the existing stage, roster, summary, waiting-room, and admission-queue components. The screen hides unavailable media controls from the capability payload and renders explicit loading, waiting, ended, rejected, and unavailable states. It remains unmounted until the session transport and route contract are ready.

2026-09-23: Tightened the live session stage at mobile widths: participant tiles now use a compact two-column gallery, smaller mobile tile sizing, and reduced stage spacing so the four-person stage fits more comfortably without changing desktop sizing. Typecheck, focused ESLint, git diff --check, and Storybook build pass.

2026-09-23: Follow-up for the desktop Session Stage story: capped participant tile height by viewport and kept session controls fixed at the viewport bottom on desktop, and above the mobile workspace bar. Typecheck, focused ESLint, diff check, and Storybook static build pass.

2026-09-23: Replaced the uniform card height cap with participant-count-aware grid sizing. Desktop column and row counts adapt to group size; card width is derived from available viewport height and row count, preserving the 16:9 aspect ratio. Added Solo, Pair, and Larger Group Storybook examples. Typecheck, focused ESLint, diff check, and Storybook build pass.

2026-09-23: Final stage layout pass: stage now fills the usable viewport with the participant gallery centered in the remaining space; live-only 'Live session' and 'Live now' labels were removed; video-disabled participants show an avatar with a camera-off marker. Added CameraOff Storybook variant. Typecheck, focused ESLint, diff check, Storybook build pass.

2026-09-23: Updated ParticipantTile presentation for avatar-only, circular camera, screen-share rectangle with avatar, and screen-share rectangle with camera PiP states. Added optional camera/screen-share MediaStream inputs and a dedicated Sessions/Participant Tile Storybook entry with permission-free mock feeds. Focused ESLint, typecheck, git diff --check, and Storybook static build pass; live RTC caller wiring remains outside this UI slice.

2026-09-23: Visually inspected Larger Group in Storybook at 1440x960. Capped desktop avatar circles and lifted them slightly so participant names no longer collide; moved speaking emphasis from the whole transparent tile to the avatar ring; fixed action-menu contrast for avatar-only tiles while retaining white controls over screen-share previews. Confirmed the updated render in Storybook; focused ESLint, typecheck, and git diff --check pass.

2026-09-23: Refined participant tile composition to keep the avatar, name, and state together as one centered group. Moved the muted mic into a circular badge centered on the avatar bottom edge; moved participant actions from the name row to a round settings button centered on the top edge, revealed on hover/focus for hover-capable devices and available by default on touch devices. Verified Larger Group at mobile Storybook viewport; focused ESLint, typecheck, diff check, and Storybook build pass.

Approved by user: implement a single-use ticket flow across backend and frontend. Access tokens stay server-side; browser receives only a short-lived single-use ticket. Code graph MCP was unavailable, so source tracing fell back to direct inspection. Existing frontend worktree clean after prior scoped commits; leave unrelated worktree changes untouched.

2026-09-24: Implemented the API-issued ten-second single-use ticket flow. Redis stores only SHA-256 keyed minimal verified principal data and consumes it atomically; native upgrade/header and auth-frame bearer support remains. Added the exact browser BFF POST route, regenerated OpenAPI/AsyncAPI and frontend types, and mounted the authenticated session signaling route with server-loaded capabilities, join/leave, stable-ID roster reconciliation, reconnect, waiting/retry, and auth failure states. Session detail lookup is manager-only and the roster contract supplies IDs only, so the page uses a generic localized session label and stable ID-derived participant labels. Validation: API/Redis cargo check, frontend typecheck and changed-file ESLint, backend fmt, git diff --check, API spec export, strict docs build passed. Full workspace lint still reports only the existing unused CardHeader import in features/workspace/components/schedule-panel.tsx; the workspace build stalled in mediasoup-sys with no running process, so package-scoped cargo check was used.

2026-09-24 follow-up: added frontend unit tests under lib/websocket/tests and features/sessions/tests covering ticket-only browser auth, reconnect ticket renewal/rejoin, unauthorized UX, stable roster snapshot/event reconciliation, waiting without media controls, and block/auth states. Full frontend Vitest: 8 files, 17 tests passed; focused ESLint for both new test files passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added single-use Redis-backed WebSocket tickets across the API and frontend, regenerated REST/AsyncAPI contracts, and wired the session route to direct API signaling with reconnect, roster reconciliation, capabilities, and waiting/auth states. Verified with API/Redis cargo check, frontend typecheck and focused ESLint, backend fmt, API contract export, docs build, and diff check.
<!-- SECTION:FINAL_SUMMARY:END -->
