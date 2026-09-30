---
id: TASK-167
title: Expose recording exports and visual stories
status: Done
assignee:
  - '@codex'
created_date: '2026-09-29 09:41'
updated_date: '2026-09-29 13:09'
labels: []
dependencies:
  - TASK-60
priority: medium
ordinal: 175000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add room-session recording list, download/retry states, English/Persian copy and Storybooks.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Members can download ready MP4s; unauthorized users cannot obtain URLs; UI states and scenes have stories.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Expose generated OpenAPI export contracts, authenticated BFF routes and a room-session recording menu with truthful polling/status/progress/download/retry; add English/Persian copy and Storybook coverage for states and actual replay scenes.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
143 frontend tests and 150 API tests pass. Storybook build and generated REST contracts pass; final browser verification and production webpack build in progress.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added recording export status/progress/download/retry dialog to room session actions, generated REST contracts and BFF routes, English/Persian copy, ten lifecycle stories and five actual replay-scene stories. Validated frontend tests, lint/type checks, production webpack build, Storybook build and seven real Chromium story checks.
<!-- SECTION:FINAL_SUMMARY:END -->
