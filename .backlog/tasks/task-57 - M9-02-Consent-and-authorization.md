---
id: TASK-57
title: M9-02 Recording notice and authorization
status: Done
assignee:
  - '@codex'
created_date: '2026-09-08 00:32'
updated_date: '2026-09-28 15:52'
labels: []
milestone: m-9
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 65000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Recording requires explicit state and authoritative permissions. Implementation scope: Recording-specific manager/admin/moderator capability with target-session access, participant notice, default-on scheduled preference, immutable control audits. No consent gate or teacher-specific role.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Unauthorized recording commands fail; every participant sees current state and failures; Start/Pause/Resume/Stop is audited.
- [x] #2 Storybook covers recording lifecycle, permissions, failures, scheduled preference and Persian/RTL states; static build and relevant frontend checks pass.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Add default-on auto_record session preference, recording-specific manager/admin/moderator capability with target-session access, participant notices, live Start/Pause/Resume/Stop controls, and immutable audits. No consent or teacher-specific role. Update roadmap and policy wording. Add Storybook stories through the production recording-controls view, including interactive lifecycle, loading/pending/failure/member states, Persian/RTL, narrow layout, session integration and scheduled preferences. Validate component regressions, typecheck, lint, static Storybook build and rendered stories.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented default-on scheduled-only auto_record with durable race protection, recording-specific manager/admin/moderator authorization with target-session access, immutable Start/Pause/Resume/Stop audits, REST status/control contracts, and English/Persian participant notices. Admitted participants retain notices even if capability lookup is unavailable; controls remain deny-by-default. Moderators gain recording control without general session management. Production capture and finalization are now wired, superseding the earlier foundation-only notes.

Validation: 150 API tests, 117 application tests, 91 domain tests; real PostgreSQL recording/session integration; 135 frontend tests, typecheck, ESLint, formatting and production webpack build; strict backend clippy/build, SQLFluff, Buf and strict documentation checks. No deployment or authenticated browser recording session was exercised.

Reopened for the requested M9 Storybook coverage. Existing recording behavior and acceptance criterion remain complete.

User requested a visual redesign and chose a toolbar recording menu with a small status indicator visible to everyone. Replace the full-width recording strip; preserve recording policy and lifecycle behavior. Update standalone/session Storybook stories and rendered interaction checks to the chosen placement.

Replaced the full-width recording strip with a compact indicator in the session toolbar and an upward-opening status/control menu. All participants can inspect status and capture details; commands remain capability-gated. Menu remains open during commands, preserves retry UUIDs, supports keyboard/Escape focus, and keeps status/error announcements accessible. Added 36 stories across toolbar lifecycle, permissions, failures, scheduled preferences and integrated session views, with English/Persian, RTL and narrow layouts. Fixed dialog portal direction for Persian. Validation: 136 frontend tests across 54 files, typecheck, full ESLint, formatting, static Storybook build and documentation build pass. Local Chromium rendered all 36 stories and exercised their play functions, including the full command lifecycle. Inspected desktop, narrow Persian, scheduled Persian and integrated-session screenshots. Browser checks use Storybook fixtures; no authenticated live recording session or deployment is claimed.

Follow-up: replaced native auto-record inputs in CreateSessionForm and SessionSettingsDialog with a shared shadcn/Radix Checkbox, semantic theme tokens and explicit accessible labels. Added the Radix checkbox dependency and jsdom ResizeObserver test stub. Six focused component tests, typecheck, scoped ESLint/formatting, static Storybook build and all eight scheduled-preference Chromium stories pass, including opt-out and Persian rendering.

Toolbar refinement: recording trigger is now icon-only with an accessible status label, full menu details and a compact warning badge. Active recording dot pulses with motion-safe animation; paused and reduced-motion states stay steady. Removed obsolete short status copy. Five component tests, typecheck, scoped ESLint/formatting and static Storybook build pass. Chromium verified five relevant stories, including icon-only presentation, active pulse, paused state, reduced motion, Persian/narrow layout and interactive lifecycle.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added authorized recording controls, scheduled recording preference and localized participant notices backed by durable audits. API/authorization, PostgreSQL and frontend regression checks pass; production capture/finalization integration is complete.

Moved recording into the session toolbar with a compact participant-visible status and contextual menu. Added 36 recording/preference/session Storybook stories, verified in Chromium with interactive lifecycle and RTL coverage. All 136 frontend tests, typecheck, lint, formatting, static Storybook and docs builds pass.
<!-- SECTION:FINAL_SUMMARY:END -->
