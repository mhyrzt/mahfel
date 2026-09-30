---
id: TASK-161
title: Align frontend AGENTS.md and DESIGN.md with the implemented design
status: Done
assignee: []
created_date: '2026-09-28 15:35'
updated_date: '2026-09-28 15:39'
labels:
  - frontend
  - docs
dependencies: []
priority: medium
ordinal: 169000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
frontend/AGENTS.md and frontend/DESIGN.md have drifted from the implemented application. AGENTS.md still lists speculative feature/lang modules (auth/meeting/ai features, missing auth/i18n libs) and treats RTC and i18n as unbuilt. DESIGN.md is raw HTML, claims a bespoke Mahfel icon set and a components/mahfel layer that do not exist, and presents unimplemented areas (Parto/AI, presence, command palette, quality diagnostics) as current. Both files misstate the theme/i18n mechanism. Update both so they describe the implementation that actually exists and clearly separate future direction.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 AGENTS.md reflects the real feature/, lib/, and app/ layout and the implemented RTC, collaboration, i18n, and appearance mechanisms
- [x] #2 DESIGN.md is valid Markdown and its tokens, fonts, radius, icons, component ownership, and theme mechanism match app/globals.css and components/ui
- [x] #3 Unimplemented design areas are labeled future direction rather than described as current
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Frontend/docs only. AGENTS.md fully rewritten to the implemented layout (features account/appearance/chat/files/i18n/invites/landing/notes/recordings/rooms/sessions/shell/whiteboard/workspace; lib api/auth/collaboration/i18n/rtc/websocket) and now reflects implemented WebRTC/mediasoup, Yjs collaboration, cookie-based i18n, data-theme appearance, WebSocket tickets, and generated contracts. DESIGN.md converted from raw HTML to Markdown and aligned to app/globals.css tokens, lucide icons, features/*/components ownership, and the data-theme mechanism; unimplemented areas (Parto/AI, presence, command palette, shortcuts, quality diagnostics, custom icon set, general toast system) are labeled Future direction. Validation: bunx prettier --check AGENTS.md DESIGN.md passes; just docs (mkdocs --strict) passes.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Rewrote frontend/AGENTS.md to the implemented architecture and stack (real feature/lib/hooks layout, implemented RTC/collaboration, cookie-based i18n, data-theme appearance, WebSocket tickets, generated contracts, scoped validation) and converted frontend/DESIGN.md from raw HTML to Markdown aligned with app/globals.css tokens, lucide icons, feature-owned components, and the data-theme mechanism, with unimplemented areas explicitly marked Future direction. Verified with prettier --check and just docs (mkdocs --strict).
<!-- SECTION:FINAL_SUMMARY:END -->
