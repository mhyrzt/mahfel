---
id: TASK-183
title: Polish session connection hover card
status: Done
assignee:
  - '@codex'
created_date: '2026-10-04 12:12'
updated_date: '2026-10-04 12:48'
labels:
  - frontend
dependencies: []
ordinal: 191000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Replace the connection-details modal with an anchored shadcn hover card, improve RTL alignment and metric typography, use a green Lucide check for connected state and animated spinner for reconnecting, and keep ms untranslated.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Connection details open without a modal overlay on hover, focus and tap
- [x] #2 Connected uses a green Lucide check; reconnecting uses animated spinner; ms remains unchanged in both languages
- [x] #3 RTL, mobile, keyboard and light/dark presentation are verified
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Use the shadcn Radix hover-card primitive to match the existing stack, refine status icon/text and card hierarchy, isolate numeric values in LTR spans, update focused tests, and verify Storybook in English/Persian and desktop/mobile sizes.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Replaced the modal with a reusable shadcn-style Radix hover-card primitive matching the existing Radix stack. Added @radix-ui/react-hover-card 1.1.23; no other packages changed. Healthy status uses a green Lucide CircleCheck; connecting/reconnecting uses LoaderCircle with motion-safe spin. Added hover, focus, click/tap, Enter and Escape access with labeled region and expanded/controls relationships; no overlay or focus trap. Card uses a status heading, three aligned metric columns, short labels and reduced explanatory copy. Units remain literal ms and values have LTR isolation; container uses RTL direction, text-start, logical borders and logical-end anchoring. Validation: 14 focused status/session-client tests passed, final status tests (5) passed, frontend lint/typecheck and scoped formatting/diff checks passed. Chromium verified 30 light/dark cases at 1280, 390 and 320 pixels, hover/focus/Enter/Escape/outside dismissal, touch opening/dismissal and reduced-motion behavior. Six final English/Persian alignment cases passed after switching to logical end alignment. Storybook remains running for review.

Measured connection quality now uses accessible face emojis: good 🙂, fair 😐, poor 🙁. Local CI verification passed frozen-lockfile installation, full formatting/lint/typecheck, all 191 tests, strict docs and the production Storybook build. Host Next build encounters an environment port-binding restriction; CI Docker runtime validation is being checked separately.

CI Docker runtime target built successfully with the pinned Bun 1.4.2, including the optimized Next production build. Host build restriction is environmental; container production packaging passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Polished session connection status with a green check, animated reconnecting spinner and untranslated ms. Replaced the modal with an anchored shadcn/Radix hover card, fixed RTL alignment and numeric ordering, and improved metric hierarchy. Focused tests, lint, typecheck, formatting and desktop/mobile browser checks pass.
<!-- SECTION:FINAL_SUMMARY:END -->
