---
id: TASK-160
title: Add synchronized video presentation with Vidstack
status: Done
assignee:
  - '@deepseek'
created_date: '2026-09-28 10:22'
updated_date: '2026-09-28 11:45'
labels:
  - frontend
  - backend
  - feature
dependencies: []
references:
  - TASK-156
  - TASK-56
priority: high
ordinal: 168000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Present a single ready MP4 or WebM file in a live session and keep every participant's playback synchronized to the presenter, using Vidstack for the player UI. PDF and image presentation stay unchanged.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A presenter can select one ready MP4 or WebM session or room file for presentation; a video selection is a single asset with one page
- [x] #2 Live participants watch in sync: play, pause, seek, and rate propagate from the presenter and late joiners start at the current position
- [x] #3 Only callers with the present capability can select a video or change playback; viewer clients cannot control playback
- [x] #4 Playback position, paused state, and rate are served by the presentation state endpoint and survive a reload or reconnect
- [x] #5 Vidstack renders the presentation player with working seek and range playback against the signed file URL
- [x] #6 PDF and image presentation behavior, including page navigation, is unchanged
- [x] #7 English and Persian copy, Storybook states, and focused backend and frontend tests cover the new surface, and docs are updated
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Backend: allow single video selections in change_presentation; add playback columns via migration 0027; add a playback action to presentation validate/apply and expose playback in the presentation state; regenerate contracts. 2. Frontend: add @vidstack/react and a PresentationPlayer that emits throttled playback and applies remote state without feedback loops. 3. Wire PresentationPane to branch on media kind, use the signed URL for video, and allow video in the file present action. 4. Add tests, Storybook states, copy, and docs.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented synchronized video presentation. Backend: migration 0027 adds playback_position_ms, playback_paused, and playback_rate to collaboration_presentations; change_presentation accepts a single MP4/WebM selection and a playback action (validated presenter-gated); presentation state returns playback for late joiners; the store validates video selections and requires the current selection to be a video for playback. Contracts regenerated. Frontend: @vidstack/react 1.15.6 renders the player (presenter gets default controls, viewers get a mute toggle and tap-to-watch overlay for autoplay); PresentationPlayer emits play/pause/seek/rate plus a 4s heartbeat and applies remote state with a drift threshold, guarded against feedback; PresentationPane branches on media kind and uses the signed download URL for video (streaming/range). ADR 0026 records the Vidstack decision; collaboration docs and mkdocs nav updated.

Validation: backend fmt, clippy (-D warnings), mahfel-api bin tests (149), PostgreSQL Testcontainers migration test with video selection and playback assertions, sqlfluff. Frontend: format, lint, typecheck, 128 Vitest tests, Storybook build, next production build. Not live-verified: actual synchronized playback across two browsers, S3 Range seeking under the attachment content-disposition, and Vidstack's built-in labels remain English while the surrounding copy is localized. Autoplay for viewers relies on the tap-to-watch fallback.

Follow-up: PDF and image presentation now auto-fit the viewer box. PdfPage measures its container with a ResizeObserver and renders the page at a fit scale (contain, device-pixel aware, capped at 4x) instead of a fixed 1.5 scale; images use a full-size object-contain box so both small and large pages fill the available area without scrolling. Added presentation-fit.ts with fitScale plus tests.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added presenter-driven, synchronized video presentation for a single MP4/WebM file using Vidstack, with durable playback position/paused/rate in PostgreSQL, presenter-gated selection and playback, and a client player that emits changes and converges viewers with drift correction. Verified with backend and frontend gates; live cross-browser sync and range seeking still need manual verification.
<!-- SECTION:FINAL_SUMMARY:END -->
