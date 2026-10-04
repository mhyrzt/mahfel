---
id: TASK-182
title: Show session connection health in the header
status: Done
assignee:
  - '@codex'
created_date: '2026-10-04 11:55'
updated_date: '2026-10-04 12:07'
labels:
  - frontend
dependencies: []
ordinal: 190000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Show compact connection health opposite the session breadcrumb, backed by actual signaling state and WebRTC transport statistics. Users can inspect latency, jitter and packet loss without crowding the stage.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Header shows signaling failures, offline state and measured media quality without fabricated measurements
- [x] #2 Details expose latency, jitter and packet loss when available and explain missing measurements
- [x] #3 English and Persian layouts work at desktop and mobile sizes; stats and lifecycle behavior have focused tests
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Add bounded local WebRTC stats sampling to the existing session client; expose transport references from media hooks; render a compact status beside the breadcrumb with a details dialog using existing Radix primitives and shared translations; test stats parsing, cleanup and connection states, then verify browser layouts.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented locale-aware session header with a connection-details dialog. Real session wiring uses signaling status, online/offline events and three-second local stats reads from existing send/receive transports. RTT uses the selected ICE pair; receive loss uses interval counters; send loss uses recent RTCP fractionLost. Quality thresholds and missing-stat behavior are documented in frontend/lib/rtc/README.md. Polling avoids overlap and ignores results after cleanup. Validation: frontend lint and typecheck passed; targeted session/RTC/shell tests passed after correcting a new test expectation and its fixture type; formatting and git diff checks passed; just docs passed. Chromium verified six connection scenarios in English/Persian at 1280 and 390 pixels, no header overlap or horizontal overflow, 40px trigger, keyboard opening/Escape/focus restoration, and dialog bounds. Storybook measurements are fixtures; actual media-session measurements were not verified end-to-end. Storybook remains running on port 6006 for review.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added connection state and estimated quality opposite the session breadcrumb, with media RTT, jitter and packet loss in an accessible localized details dialog. Uses browser transport stats without extra network probes, preserves unavailable measurements and prioritizes offline/reconnecting/interrupted states. Local tests, checks and browser previews pass; live media-session verification remains unperformed.
<!-- SECTION:FINAL_SUMMARY:END -->
