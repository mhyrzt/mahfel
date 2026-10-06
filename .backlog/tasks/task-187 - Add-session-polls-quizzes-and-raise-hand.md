---
id: TASK-187
title: Add session polls quizzes and raise hand
status: Done
assignee:
  - '@codex'
created_date: '2026-10-06 07:35'
updated_date: '2026-10-06 11:29'
labels: []
dependencies: []
documentation:
  - docs/architecture/session-interactions.md
ordinal: 195000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement the approved session interaction plan: single/multiple-choice polls with changeable votes, quizzes with multiple correct answers and exact-set grading revealed only on close, creator-selected anonymous/named voting, durable results and an ordered shared hand queue. Polls and quizzes occupy a stage view alongside whiteboard and presentation, with one question per slide and query-preserved navigation. Use the outlined hand icon per the final user correction, a neighboring emoji reaction menu and transient username-plus-emoji animations. Audit and fix Vazirmatn loading and font inheritance.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Polls persist single/multiple votes with respondent-based result bars and change/retract while open
- [x] #2 Quizzes hide answer keys and distributions until closing, grade exact sets and permit only one final submission
- [x] #3 Anonymous submissions never disclose voter choices; named results follow reveal policy
- [x] #4 Raise hand shares one ordered entry across devices, supports moderation and clears on departure/session end
- [x] #5 Session UI uses adapted shadcn Questionnaire and supports English Persian RTL themes keyboard and mobile
- [x] #6 Generated contracts docs and focused backend/frontend integration checks cover retries races and recovery
- [x] #7 Live reactions validate current participation, rate-limit across devices and replicas, and show bounded username-plus-emoji upward animations with reduced-motion support
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Inspect existing session authorization persistence and delivery seams. 2. Add domain/application/PostgreSQL interactions and lifecycle cleanup with atomic commands. 3. Add authenticated REST and versioned session interaction notifications on existing WebSocket with snapshot recovery. 4. Adapt shadcn Questionnaire and integrate activity panel creation/result flow and hand queue. 5. Generate contracts test behavior and RTL responsive stories update docs and finalize.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented PostgreSQL-owned activities, responses, ordered hands and durable command retry fingerprints; REST redaction and named voter pagination; metadata-only WebSocket invalidations with pub/sub wakeups and two-second version polling; shadcn Questionnaire panel and creation form, toolbar hand control, roster queue/indicators and ended-session history. Anonymous choices are hidden from other participants and managers, but stored against actors for uniqueness. Quiz exact-set grading and reveal-on-close enforced server-side. Source, type, component and transport checks pass. Final visual/browser and local-network verification remain pending because automatic approval review failed with HTTP 404. Standard Next production build cannot bind its helper port in this sandbox; Webpack fallback yields empty TypeScript --showConfig output from the detached subprocess. A direct TypeScript invocation succeeds. No changes made to Next build configuration to hide the failure. The real disposable PostgreSQL interaction scenario passed before the final membership-access hardening; its expanded test compiles but needs a container rerun. Keep task In Progress until final verification.

User expanded scope during implementation: moved activities from sidebar into the stage selector; removed the earlier activity sidebar integration; added a reaction button beside the filled hand control. Eight emoji reactions use authenticated REST, Redis fixed-window actor/session counters and session-scoped pub/sub events. They intentionally are transient and do not replay, unlike durable votes/hands. Frontend caps animations at 12 and seen event identifiers at 128 with 3.5-second removal, supports reduced motion and isolates usernames in RTL. Found Storybook missing the Vazirmatn CSS import while production root already imports it; corrected preview and shared Tailwind sans-family token. Preserved explicit session admission for private-room guests. Updated contracts and docs. Latest focused frontend check: 24 files / 72 tests passed; Rust scoped clippy passed. Final live Redis/browser and expanded PostgreSQL rerun remain pending approval service recovery.

Expanded-scope checks passed: scoped Rust clippy; 48 API unit tests including quiz redaction, bounded reaction palette, scoped notification/backpressure and two-replica polling recovery (Redis Testcontainers-gated pre-existing test does not run live without MAHFEL_TESTCONTAINERS=1). Latest frontend focused suite 24 files / 72 tests passed; frontend typecheck, lint, formatting and Storybook build passed; strict docs build passed. Earlier full frontend suite 67 files / 208 tests passed before the reaction/stage/font expansion. TASK-187 stays In Progress for actual browser font/RTL/mobile checks, full local-network API tests, live reaction pubsub checks and expanded disposable PostgreSQL rerun; approval review remains unavailable with HTTP 404.

Changed multiple questions to a slideshow per user correction: one visible activity slide, Previous/Next with boundary disabling, localized question position and mirrored RTL arrows. Hidden slides retain unsent choices while navigating, and selection tracks activity identity across snapshot updates. Applied to both live activity view and ended-session history. Added slideshow behavior/RTL tests. Validation passed: 18 interaction tests, frontend typecheck, scoped ESLint, Prettier, git diff check and strict docs build. Existing browser/live verification blockers remain unchanged.

Applied session layout correction: removed transformed stage ancestry from control positioning and placed toolbar in a reserved bottom flex row; made sidebar inset a full-height flex container. Activities use the full remaining height, center the question, scroll overflow independently, and place larger Previous/Next controls below the slide. Polished question typography and card spacing. Breadcrumb now includes the localized active view. Native Next-integrated history preserves view and question query parameters, unrelated query values and fragments without reconnecting the session; activity slides stay mounted while changing views to retain unsent selections in memory. Reloads restore view/question, not unsent selections. Validation: 35 focused frontend tests passed; TypeScript, scoped ESLint, formatting, Storybook build, strict docs and diff checks passed. Production build still fails because the sandbox denies the Turbopack helper port. Browser visual validation remains pending approval-service recovery; task remains In Progress.

Replaced modal draft creation/editing with an embedded composer inside the activities stage. The form body scrolls within its own div while the header and save footer remain visible. Compact poll/quiz selection tiles, numbered option rows with inline correct-answer checks, and a two-column settings area preserve RTL and accessible labels. Slides remain mounted but hidden during composing; cancelling restores the selected slide and unfinished selections. Removed obsolete composer modal props and updated stories/tests. Fixed missing toolbar controls in generic previews: live SessionScreen stories provide hand/reaction controls, and the toolbar shows disabled fallback controls when handlers are absent rather than omitting them. Focused tests pass: 33 tests across 6 files; scoped lint, TypeScript and strict docs pass. Browser visual checks remain blocked by the approval-service error; overall task stays In Progress.

Completed another scoped visual/source audit from user screenshots. The standalone toolbar story was rendering disabled fallback controls without handlers; its default Interactive story now supplies stateful media and hand handlers, local reaction events, and expiring username/emoji animations. Shared preview behavior also powers session-screen stories, removing inert static controls. Added distinct hand-raised, media-active, media-unavailable (interactions still enabled), interaction-loading/unavailable, listen-only, poll-view, host, Persian and narrow Persian states. Toolbar items no longer shrink away on narrow screens and scroll within parent width. Reverted all hand icons to outline per user correction. Activity status uses open/closed lock icons with localized accessible labels; anonymous voting uses VenetianMask. Composer settings now use two full-width rows with matching label/switch order, avoiding unequal-height columns; privacy help is associated with its switch. Focused checks passed: 35 tests plus the added media-unavailable interaction regression; scoped ESLint, TypeScript, docs and diff checks passed. Browser visual checks remain pending approval-service recovery.

Release validation now has working network/Docker approval. All 401 backend unit tests, Clippy, rustfmt, migration SQL lint, doctests, strict docs, Buf checks, frontend lint/format/type checks, Storybook and the production web image passed. Disposable PostgreSQL interaction and migration tests passed, along with Redis replica recovery checks. The integration run passed 427 tests with four Keycloak startup timeouts; three passed when rerun using the CI profile, while the fourth reached its container startup limit during the cold native image build. A fresh frontend suite and that startup test will run after packaging finishes. Production web and API health endpoints currently return HTTP 200. Browser inventory contains no connected surfaces; manual visual and authenticated browser E2E remain unverified. Added targeted Next.js 16.3.6 and runtime OpenSSL patches after local Trivy findings; rebuilt web image has zero HIGH/CRITICAL vulnerabilities. Split commits preserve unrelated README edits.

Final local release gates passed: full frontend suite 225 tests across 72 files with one worker; backend unit suite 401 passed; integration suite 427 passed plus four startup-limited Keycloak cases passed on CI-profile reruns after native compilation. Rustfmt, Clippy, SQLFluff, doctests, frontend format/lint/typecheck, Storybook, strict docs, Buf, deployment script checks and six deploy tests passed. API/media/worker production runtime images built successfully. Web runtime health passed; Trivy reports zero HIGH/CRITICAL. Targeted API Perl patch removes three CRITICAL and four HIGH findings; remaining advisory scan reports 1 CRITICAL and 61 HIGH with no listed fixes. No connected browser surface was available, so manual visual and authenticated multi-user browser E2E are not claimed. User-authorized split commits and protected component releases follow these local checks.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented durable polls/quizzes, slideshow activity stage, query-preserved views, embedded composer, ordered hand queue, rate-limited emoji reactions and interactive RTL Storybook states. Validated full frontend and backend checks, real disposable PostgreSQL/Redis behavior and production image builds; patched available Next/OpenSSL/Perl vulnerabilities. Browser visual/E2E verification remains unclaimed; remaining Debian scan findings are advisory.
<!-- SECTION:FINAL_SUMMARY:END -->
