---
id: TASK-126
title: M4-10 Frontend session state and roster
status: In Progress
assignee:
  - '@mahyar'
created_date: '2026-09-20 18:58'
updated_date: '2026-09-22 22:59'
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
Client-side generic session state for the browser app: connect the WebSocket with a verified token, join and leave a session, render a roster from stable participant identities, and reconcile state across reconnects. Waiting or lobby state from admission policy is represented and hides media controls until admitted. Capability payload gates controls the caller cannot use. Education-facing copy is theming, not core.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The roster reflects join, leave, remove, and block events and survives reconnect without duplicate participants.
- [ ] #2 Reconnect re-establishes signaling and reconciles the stable participant identity.
- [ ] #3 Lobby policy shows a waiting state and no media controls before admission.
- [ ] #4 Controls the caller cannot use are hidden or disabled from the capability payload.
- [ ] #5 Expired or unauthorized tokens produce actionable UX instead of silent failure.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Export the code-first mahfel-api OpenAPI 3.1 document to openapi/mahfel-api.json, then generate frontend/lib/api/generated/schema.ts with openapi-typescript.
2. Add a small Axios client constrained to the same-origin authenticated BFF and a typed error boundary; do not expose Keycloak tokens to browser code.
3. Add feature-scoped session API adapters and a controlled session state model with loading, waiting, live, ended, rejected, and unavailable states.
4. Compose the route-level session screen from the existing roster, waiting-room, stage, toolbar, and moderation components; use fixtures until the incomplete M4 REST and WebSocket contracts are ready.
5. Add focused reducer/component tests and stories; validate formatting, lint, typecheck, and relevant frontend tests.
6. Integrate only after explicit BFF allowlist and browser-safe WebSocket credential/roster-snapshot contracts are available.
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
<!-- SECTION:NOTES:END -->
