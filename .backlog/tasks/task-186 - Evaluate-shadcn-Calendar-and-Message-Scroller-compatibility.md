---
id: TASK-186
title: Evaluate shadcn Calendar and Message Scroller compatibility
status: Done
assignee:
  - '@codex'
created_date: '2026-10-05 10:26'
updated_date: '2026-10-05 10:52'
labels:
  - frontend
  - ui
dependencies: []
documentation:
  - doc-1
ordinal: 194000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Evaluate the latest official Calendar and Message Scroller against existing Persian/Gregorian UTC and reader-controlled chat-scroll behavior before introducing dependencies.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Calendar and chat scrolling receive evidence-backed compatibility decisions preserving existing behavior
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Review fetched Calendar and Message Scroller sources and underlying APIs, compare Persian/Gregorian UTC and history-scroll contracts, record go/no-go decisions before adopting.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Reviewed official Message Scroller headless API and fetched wrappers. Reviewed current DayPicker v10 Persian/time-zone docs and compatibility migration from v9. Calendar is compatible in principle but not a drop-in: standard shadcn wrapper and Persian build must be adapted with UTC/date-key parity; retain the current calendar with shadcn Buttons for this delivery. Message Scroller is compatible in principle (autoScroll while at live edge, default end, stable message IDs), but requires replacing scroll ownership and browser parity for history/resize/session changes. Browser tool inventory is empty and iab creation reports unavailable; retain current scroll behavior rather than assert untested parity.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Reviewed current official wrappers and underlying APIs. Retain the custom calendar with shadcn day Buttons and existing chat scrolling: both replacements require explicit adaptation and browser parity verification. Decisions and official sources recorded in doc-1; no untested dependency introduced.
<!-- SECTION:FINAL_SUMMARY:END -->
