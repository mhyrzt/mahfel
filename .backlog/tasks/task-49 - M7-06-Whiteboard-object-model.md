---
id: TASK-49
title: M7-06 Whiteboard object model
status: Done
assignee:
  - '@codex'
created_date: '2026-09-08 00:31'
updated_date: '2026-09-28 08:41'
labels: []
milestone: m-7
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 57000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Canvas collaboration needs compact mergeable semantic objects.

Implementation scope: Object schema/versioning, create/update/delete ordering, transform batching, viewport and basic shapes.

Dependencies: M7-02–M7-04
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Two browsers edit objects concurrently and converge; reload preserves board.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Store Excalidraw elements in a Yjs map with deterministic ordering, sync changes, share viewport, and record presentation events for stored PDFs and images.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Design note: Whiteboard pan and zoom are shared room state: a participant changing the viewport moves it for everyone, with competing actions resolved by latest accepted event. Keep viewport events distinct from semantic drawing edits. A presenter-controlled slideshow can reference stored PDFs or ordered images and record slide changes.

Implementation present. Validation: backend Clippy and formatting, PostgreSQL migration integration, API unit suite, frontend typecheck, lint, formatting and 19 tests, production webpack build. Acceptance needs live multi-client/restart verification before marking done.

Embedded PNG/JPEG images now use verified session-file uploads, document-scoped registration, PostgreSQL delete protection, and stable Yjs file references. Lexical notes and Excalidraw whiteboard load bytes through authorized presigned downloads. Backend unit/integration and frontend unit checks pass; live two-browser convergence/restart acceptance remains open.

2026-09-26: Stopped applying local Yjs map changes back into Excalidraw during drawing; editing now waits for socket permission. Added awareness cursor relay and restricted viewer input/viewport updates in both client and PostgreSQL store. Source checks pass; two-browser convergence and reload acceptance remain unverified.

2026-09-26: Presentation pane now offers direct session-file upload, and local Caddy adds CORS for the presigned S3 host. The prior API presentation error mapping was corrected in this working tree. Live upload and display still need a rebuilt stack and browser validation.

2026-09-28: User confirmed M7 implementation, testing, and refinement are complete and requested milestone closure. Acceptance closed on that confirmation; no tests rerun for this bookkeeping update.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Completed as part of M7 collaborative notes and whiteboard. Implementation, testing, and refinement confirmed complete by the user on 2026-09-28.
<!-- SECTION:FINAL_SUMMARY:END -->
