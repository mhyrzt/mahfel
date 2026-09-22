---
id: TASK-137
title: Frontend i18n foundation for English and Persian
status: In Progress
assignee:
  - '@mahyar'
created_date: '2026-09-22 20:04'
updated_date: '2026-09-22 21:56'
labels:
  - frontend
dependencies: []
priority: medium
ordinal: 145000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Provide a typed, server-safe localization foundation for the Mahfel web application with English and Persian support, correct document direction, and a persisted user language preference. Feature UI can migrate incrementally without changing API contracts.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 English and Persian are supported locales with English as a validated fallback.
- [ ] #2 The rendered document uses the selected locale and correct LTR or RTL direction.
- [ ] #3 Users can change the locale and the preference persists across page refreshes.
- [ ] #4 Message catalogs are typed and tests cover locale validation and direction.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Establish typed en/fa locale configuration, server lookup, and persisted user preference with document direction. 2. Migrate the existing session components and state-driven SessionScreen to typed localized messages. 3. Add English and Persian Storybook stories with lang/dir wrappers for session states and components. 4. Run frontend typecheck, lint, and Storybook build; record progress and remaining migration scope.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
2026-09-22: Added typed en/fa message catalogs, validated locale and direction helpers, server-side locale resolution from NEXT_LOCALE, and a language switcher that persists the preference in a SameSite=Lax one-year cookie. Root layout now emits matching lang and dir attributes; the home page is the first migrated consumer. Focused tests and frontend typecheck pass.

2026-09-22: Migrated all existing session UI copy, roles, and accessible labels to the typed English and Persian catalogs. Added Persian RTL stories for the roster, admission queue, waiting panel, and stage, plus SessionScreen stories for loading/waiting/live/ended/rejected/unavailable states. Storybook static build and changed-file ESLint/typecheck pass. Full frontend lint still reports two existing workspace errors in room-list.tsx and schedule-panel.tsx.

2026-09-23: Reserved `/` as the public product landing and information page. Middleware now allows root access without a session, including when a stale/invalid auth cookie is present; other route policies remain unchanged. Organized all stories under feature-level `stories/` folders (and `components/ui/stories`); existing Storybook globs discover the new locations.

2026-09-23: Localized the workspace shell navigation for English and Persian, added a responsive mobile bottom bar with a create-action sheet and safe-area spacing, and added an RTL mobile shell Storybook story. Installed @fontsource-variable/vazirmatn and applied it to lang=fa. Typecheck, focused ESLint, git diff --check, and Storybook build pass.
<!-- SECTION:NOTES:END -->
