---
id: TASK-181
title: Fix Persian shell sidebar overlap
status: Done
assignee:
  - '@codex'
created_date: '2026-10-04 11:49'
updated_date: '2026-10-04 11:54'
labels:
  - frontend
  - bug
dependencies: []
ordinal: 189000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
AppShell gives its fixed rail a locale direction but lets content spacing inherit an unrelated ancestor direction. Persian previews can reserve space on the left while the rail covers content on the right.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Shell rail and reserved content space follow the shell locale regardless of ancestor direction
- [x] #2 Persian and English desktop and mobile layouts are verified
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Set locale direction on the shell root using the shared helper; add desktop and mobile stories without external direction decorators; validate computed browser geometry and frontend checks.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
AppShell now applies directionForLocale(locale) at its root so the fixed rail, border and logical content padding share direction. Added Storybook geometry assertions and separate Persian desktop/mobile stories; mobile stories use current viewport globals. Validation: frontend lint, typecheck, scoped Prettier, git diff --check and 16 shell tests passed. Headless Chromium verified shell and live-session bounds in English/Persian at 1280, 768 and 390 pixels with both LTR and RTL ancestors: 24 cases passed, no horizontal overflow. Persian desktop content spans x=0..1216 and rail x=1216..1280. Verified locally in Storybook; no production deployment performed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Fixed Persian sidebar overlap and empty left rail space by setting direction at the AppShell root. Added browser geometry regression assertions and desktop/mobile Persian stories. Local browser layout checks and frontend validation pass.
<!-- SECTION:FINAL_SUMMARY:END -->
