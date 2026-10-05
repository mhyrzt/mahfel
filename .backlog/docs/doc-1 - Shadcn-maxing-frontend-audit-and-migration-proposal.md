---
id: doc-1
title: Shadcn maxing frontend audit and migration proposal
type: specification
created_date: '2026-10-05 10:19'
updated_date: '2026-10-05 13:47'
---
# Shadcn maxing: frontend audit and migration proposal

Date: 2026-10-05. Status: approved by the user and applied; pinned-Bun Docker production packaging and health verified; browser verification remains incomplete. The audit census below records the pre-migration state.

## Objective and policy

Use locally owned shadcn components for ordinary UI controls, including simple inputs, textareas, labels, checkboxes, radios, switches, selects, buttons, tooltips, and feedback. Compose feature UI from these primitives while retaining Mahfel's design tokens, density, Vazirmatn typography, light/dark/system themes, Persian/English support, and domain behavior.

Native HTML inside `components/ui` is expected. Semantic layout elements remain HTML. The target is complete coverage of eligible product controls, not replacing every HTML element with a component or installing the whole catalog.

## Evidence and limits

- Read repository/frontend instructions, `frontend/DESIGN.md`, ADR 0014, package/configuration files, shared primitives, relevant feature implementations, and Backlog tasks 153, 156, and 158.
- Codebase graph lookup reported Mahfel is not indexed; direct source discovery was used.
- Census: 111 TSX files under `frontend/features` and `frontend/app`, excluding directories named `tests` and `stories`. Counts are JSX source sites, not rendered instances or defect counts. It excludes shared primitives, dependency internals, and browser-generated DOM.
- There are 23 top-level UI modules. `Button` is imported by 44 product files. `Input`, `Textarea`, `Checkbox`, and `Select` already exist; textareas have no native product sites.
- Source census found 12 native button sites, 8 input sites, 2 select sites, 18 label sites, 1 progress site, and 1 table site. Of 8 input sites, 4 are file pickers, 2 checkboxes, 1 switch implemented with a checkbox, and 1 range input.
- This is a source and upstream-registry audit. No running app, screenshots, keyboard session, accessibility tree, media playback, or live network flow was tested. Test results in old Backlog notes are historical, not checks run for this audit.
- Existing `README.md` edits are user-owned and outside this proposal.

## Current official shadcn snapshot

Official [component catalog](https://ui.shadcn.com/docs/components), [changelog](https://ui.shadcn.com/docs/changelog), [CLI](https://ui.shadcn.com/docs/cli), [RTL guide](https://ui.shadcn.com/docs/rtl), and Radix docs were fetched live. The CLI resolved to **4.21.1**. Registry definitions were fetched with read-only `view` commands into `/tmp/mahfel-shadcn-registry.json`; no components were installed. A second read-only snapshot at `/tmp/mahfel-shadcn-registry-extra.json` contains Button, Input, Textarea, Checkbox, Slider, Collapsible, Tooltip, Separator, Breadcrumb, Calendar, AlertDialog, Sonner, and Message Scroller. Together, the two snapshots contain 27 registry definitions.

The fetched `new-york-v4` definitions cover Field, Label, Switch, Radio Group, Toggle, Toggle Group, Native Select, Input Group, Progress, Table, Empty, Alert, Spinner, and Direction. These are directly relevant to the gaps below.

Official documentation references:

- [Field](https://ui.shadcn.com/docs/components/radix/field): labels, descriptions, validation, and semantic field grouping; does not require adopting a form library.
- [Radio Group](https://ui.shadcn.com/docs/components/radix/radio-group) and [Toggle Group](https://ui.shadcn.com/docs/components/radix/toggle-group): choose based on selection semantics.
- [Native Select](https://ui.shadcn.com/docs/components/radix/native-select): shadcn coverage can preserve the existing native select behavior.
- [Input Group](https://ui.shadcn.com/docs/components/radix/input-group): consider for the chat composer rather than repeating control styling.
- The catalog also includes Message Scroller, Questionnaire, Button Group, Empty, and Spinner. Questionnaire has no current product requirement here. Message Scroller is an evaluation candidate, not an assumed drop-in replacement. Its fetched wrapper uses `@shadcn/react/message-scroller` and exports Provider, Viewport, Content, Item, and jump-button components, plus scroll/visibility hooks. The wrapper source is reviewed; the underlying package behavior still needs verification.

Compatibility decisions before installation:

1. Preserve `new-york`, Radix, semantic tokens, and Lucide. Current documentation also offers other primitive bases; this request does not justify changing the existing foundation.
2. The fetched components import `cn` from the new package and several import unified `radix-ui`. Mahfel uses local `@/lib/utils` and individual Radix packages. Recommended first pass: adapt generated imports to the existing helper and use individual Radix packages consistently, adding only missing primitives. Treat a whole-project `cn` or unified-Radix migration as a separate decision, not an incidental CLI side effect.
3. Current registry components use `dark:` styles. Mahfel selects themes through `data-theme`, including OS-following `system`, rather than a `.dark` class. Adapt these styles to semantic tokens or a reviewed custom variant; verify both explicit dark and system-dark. Do not change theme persistence as part of this work.
4. New-york does not receive the newer styles' automatic RTL treatment just by setting `rtl: true`. Add/review DirectionProvider; explicitly review logical padding, positioning, switch thumb movement, portal direction, and arrow-key behavior. Sidebar, Calendar, and Pagination require manual review according to the RTL guide.
5. The source uses `animate-in`/`animate-out` utility classes, but the stylesheet imports only Tailwind and package.json does not list `tw-animate-css`. Verify generated CSS before claiming these transitions work. Add/adapt animation support only if needed, with reduced-motion behavior.
6. Never run global overwrite, preset application, or `add --all`. Preview generated files and transitive registry dependencies with `add --dry-run` and diff each primitive. Field depends on Label and Separator; Input Group depends on Button, Input, and Textarea, so existing customized files must be protected.

## Audit findings and replacement map

| Surface | Source evidence | Proposed shadcn composition | Risk / behavior to retain |
| --- | --- | --- | --- |
| Device preview switch | `features/account/components/device-preferences.tsx:160` | Switch + Field/Label | Controlled state; media track cleanup; device errors; keyboard and RTL |
| Admission switch | `features/rooms/room-admission-settings.tsx:49` | Switch | Pending disable, server-authoritative saved value, failure feedback |
| Permission matrix | `features/sessions/components/access-management.tsx:94,130` | Table + Checkbox | Sticky row headings, accessible participant/permission names, self/manager locks, pending state; use `checked === true` rather than treating indeterminate as a boolean |
| Recording track selection / seek | `features/recordings/recording-player.tsx:242,277` | Checkbox + FieldSet; Slider for seek | Per-track mute inversion, zero duration, 1000ms step, playing/paused updates, LTR media timeline even in Persian |
| Role and room selection | `features/rooms/room-members.tsx:153`; `features/sessions/components/create-session-form.tsx:106` | NativeSelect initially | Preserve option values, empty placeholder, pending disable, and current form behavior; use existing Select only if richer popup behavior is needed |
| Room visibility | `features/rooms/create-room-form.tsx:69` | RadioGroup + FieldSet/FieldLabel | Required exclusive choice, existing default, arrow navigation, unchanged submitted values |
| Theme / language / calendar preferences | `components/ui/segmented-control.tsx`; three feature consumers | RadioGroup under existing feature presentation | Exactly one selected preference; retain API and cookie behavior; ToggleGroup only for genuinely toggleable choices |
| Form labels and validation | create room/session, session settings, invite join, device preferences | Field, FieldLabel, FieldDescription, FieldError, Label | Stable IDs, label association, `aria-describedby`, `aria-invalid`; field errors stay distinct from server/global errors |
| Ordinary buttons / attachment actions | conversation composer, message preview tiles, session agenda triggers, upload tile, session notification | Button with reviewed variants / `asChild` composition | Preserve card/tile geometry, multiline text, event handlers and accessible names; current Button defaults include flex layout and nowrap, so do not perform blind tag replacement |
| Session files disclosure | `features/rooms/session-files.tsx:97` | Collapsible + Button | Lazy first load, count, open state, errors, and focus |
| Recording export progress | `features/recordings/recording-list-view.tsx:148` | Progress | Preserve value/max semantics and status announcement |
| Notes toolbar | `features/notes/components/notes-toolbar.tsx` | Tooltip + Separator; assess Toggle for formatting | Existing buttons already use shadcn; replace native title tooltips. Formatting toggles require actual Lexical selection state, not decorative pressed state |
| Empty/error/loading patterns | chat list, room sessions, invite list, feature error paragraphs | Empty, Alert, FieldError, Skeleton/Spinner | Keep localized text, live-region meaning, contextual recovery; preserve screen-reader-only announcements |
| Chat composer | `features/chat/components/conversation-panel.tsx` | InputGroup + Textarea + Button + Tooltip | Enter/Shift+Enter, IME composition, reply cancellation, attachment lifecycle, compact sidebar layout |
| Breadcrumb | `features/sessions/components/session-breadcrumb.tsx` | Evaluate Breadcrumb | Preserve links, current-page semantics, overflow and RTL |

### Deliberate exceptions and evaluations

- Four native file inputs: session file picker, chat composer, file library, and notes toolbar. Keep native file selection plumbing. Use shadcn Button/Label for visible triggers; use Input for a visible native file field where appropriate. Hidden/ref-driven file inputs may remain documented exceptions to avoid changing focus, drag/drop, multi-file behavior, or resetting the selected file.
- `month-calendar.tsx` implements Persian and Gregorian month arithmetic, Saturday-first layout, active dates, selected/today states, and UTC date keys. First replace the day buttons with adapted Button. The fetched official Calendar wraps `react-day-picker` and adds `date-fns`; its default source imports the standard DayPicker, not a Persian adapter. Evaluate the current official Calendar/Persian adapter in a separate slice; require parity tests before removing custom month logic or adding a calendar dependency.
- Keep Lexical, Excalidraw, PDF canvas rendering, Vidstack, native audio/video, and realtime transports. Shadcn wraps their surrounding UI; it does not replace those domain engines. The microphone level meter is a meter, not task completion; do not turn its semantics into Progress just for component coverage.
- Chat already uses Message/Bubble, avatars, menus, and a near-bottom scrolling heuristic. Evaluate Message Scroller against loading older history without moving the reader, first load, own/new messages, replies, room/session switches, sidebar resize, and explicit jump-to-latest behavior. Adopt only after reviewing the actual component API and source.
- Keep specialized dark session chrome defined in DESIGN.md. A generic component migration does not justify palette, typography, or layout changes.
- Assess AlertDialog for existing destructive confirmation flows. Keep authorization and confirmation sequencing intact. A toast replacement for the custom chat notice is optional; simple Button composition can achieve the immediate goal with less behavioral change. The fetched Sonner wrapper introduces `next-themes`; do not adopt that theme provider over Mahfel's existing cookie/data-theme system. Adapt theme integration explicitly if Sonner is chosen.

## Ordered implementation plan

### 1. Establish the shared foundation

Capture current stories/screens for light/dark/system and Persian/English. Add only the first slice's required components: Field/Label, Switch, RadioGroup, NativeSelect, Direction. Review imports, dependencies, tokens, invalid/focus styles, sizes, animations, and portal behavior. Update DESIGN.md with the maxing policy and actual primitive inventory. Keep client boundaries narrow.

Acceptance: new primitives support all theme/locale combinations, expose expected keyboard semantics, and do not overwrite customized existing components.

### 2. Migrate forms and settings

Room/session creation, invite join, session settings, member role selection, admission/device switches, and theme/language/calendar selectors. Retain existing form state and API calls. Replace bare labels with Field composition and associate errors explicitly. Remove obsolete custom switch CSS and simplify or remove SegmentedControl once all consumers migrate.

Acceptance: existing submissions, defaults, cookies, pending/error behavior, and selection values are preserved; keyboard radio navigation and label activation work.

### 3. Migrate permissions and playback controls

Permission matrix to Table/Checkbox; recording tracks to Checkbox/FieldSet; timeline to reviewed Slider; export status to Progress. Do not alter permission rules, replay timing, recording transport, or media cleanup.

Acceptance: self/manager permissions stay locked; failed updates retain saved state; recording mute/seek behavior is unchanged at zero/end duration and during playback.

### 4. Complete visible control coverage

Replace eligible remaining bare buttons, add Collapsible for session files, improve upload triggers, add notes/composer tooltips and separators, and normalize empty/error/loading feedback. Evaluate Breadcrumb and InputGroup in this slice. Preserve tile and row layouts with explicit primitive variants rather than copying styles into every consumer.

Acceptance: every ordinary native control is migrated or recorded as an intentional exception; focus, dialog triggers, upload/drop behavior, and accessible names remain correct.

### 5. Evaluate calendar and chat scrolling separately

Run the Calendar parity evaluation and Message Scroller prototype independently. Each receives a go/no-go decision based on documented compatibility and behavior. They are not prerequisites for completing the ordinary-control migration.

Acceptance: no lost Persian calendar behavior, history scroll position, session scope, or reader control. Retained custom implementations have a documented concrete reason.

### 6. Consolidate and verify

Remove migrated dead code/styles, ensure feature logic remains feature-owned, and refresh DESIGN.md inventory and exceptions. Repeat the source census, review actual rendered controls, and finalize implementation Backlog tasks only when their gates pass.

## Verification and delivery

- Per slice: formatting, lint, typecheck, and focused existing tests. Add behavior tests for changed semantics (switch pending/failure, RadioGroup keyboard behavior, permission locks, seek callback translation), not tests that merely assert wrapper markup.
- Add Storybook coverage for new reusable primitives and representative error/disabled/selected states; build Storybook with the existing accessibility addon.
- Final relevant gates: `just frontend fmt`, `just frontend lint`, `just frontend typecheck`, `just frontend test`, `just frontend storybook-build`, and `just docs`. Run `just frontend build` when new dependencies or client-boundary changes affect production packaging.
- Browser checks: Persian/English × light/dark, system following, desktop/mobile, keyboard focus/Space/arrow keys, portal direction, accessible labels/errors, dialogs, uploads, permission changes, and playback. Add focused Playwright coverage under `e2e/tests` for affected interactions where useful.
- Report source checks, local tests, Storybook packaging, browser proof, CI, and deployed proof separately. Any unavailable check remains explicitly unverified.
- Coordinate overlap with TASK-153 and TASK-156; TASK-158 is Done and documents calendar/chat/upload behavior to preserve. Do not reopen or close those tasks merely because this migration touches their files.
- This document records the requested audit/proposal. After review, create implementation tasks for slices 1–4 and a separate evaluation task for slice 5 through the Backlog CLI. Record the approved plan before coding. No implementation task is marked complete by this audit.

## Applied migration and verification (2026-10-05)

The user approved implementation. TASK-184 and TASK-185 track the shared foundation, forms/settings, permissions/playback, remaining visible controls, and consolidation; TASK-186 records optional compatibility decisions. Existing TASK-153 and TASK-156 statuses were left intact.

- Added 15 adapted primitives, bringing the top-level UI inventory from 23 to 38: Field, Label, Switch, RadioGroup, NativeSelect, Direction, Table, Slider, Progress, Collapsible, Empty, Alert, Spinner, Breadcrumb, and InputGroup. Existing Button gained a plain variant for tiles/rows, and Input gained invalid-state styling.
- Adapted current registry source to local helpers, individual Radix imports, semantic data-theme tokens, and logical RTL positioning. Added the missing individual Radix dependencies and tw-animate-css, with reduced-motion support. Added locale direction context without changing server ownership of theme/locale cookies.
- Migrated the ordinary controls and shared feedback in slices 1 through 4. Preference selectors now use radio semantics; admission changes retain saved state on failure; permissions retain self/manager locks and pending behavior. Recording seek explicitly stays LTR and forwards its accessible name to the slider thumb.
- Added representative English/Persian, invalid, and disabled form-control stories. Added behavior tests for radio arrow navigation, media seek direction/callback, progress value semantics, and failed admission/permission updates. Adapted existing tests to the new radio/upload trigger semantics.
- Updated DESIGN.md and added ESLint enforcement against raw ordinary controls in product TSX files. Native file inputs remain an explicit exception.
- Final source census: 111 product TSX files; zero raw button, textarea, select, label, progress, or table sites; four raw input sites, all native file-selection plumbing. Source counts do not prove rendered browser accessibility or visual parity.

Calendar decision: retain the current Persian/Gregorian UTC month implementation with shadcn day Buttons. The current standard shadcn Calendar is not a Persian drop-in. DayPicker v10 splits the Persian implementation into @daypicker/persian and supports explicit time zones; adaptation needs UTC date-key, Saturday-first, month-navigation, active-day, and selected/today parity. No calendar dependency was added. Evidence: [shadcn Calendar](https://ui.shadcn.com/docs/components/radix/calendar), [DayPicker Persian](https://daypicker.dev/localization/persian), [v10 upgrade](https://daypicker.dev/upgrading), [time zones](https://daypicker.dev/localization/setting-time-zone).

Message Scroller decision: retain current chat scrolling. The candidate owns follow/anchoring/history behavior; adoption would require stable message IDs, a bounded viewport, deliberate autoScroll configuration, and browser checks for history prepending, resize, new/own messages, scope changes, and jump-to-latest. This delivery reviews compatibility without claiming a tested replacement prototype. Evidence: [official Message Scroller API](https://ui.shadcn.com/docs/react/message-scroller).

Validation evidence:

- Full Vitest suite: 65 files, 197 tests passed. A final focused radio/slider/progress/account preference run passed 5 tests after the last direction/disabled changes.
- Formatting, zero-warning ESLint, strict typecheck, Storybook production build, git diff whitespace check, and just docs passed. Storybook reports bundle-size warnings; this is packaging proof, not a completed visual or accessibility audit.
- Standard Next production build failed when the Turbopack CSS worker tried to create a process/bind a port (Operation not permitted), including the escalated retry. A Webpack fallback also failed with Could not parse output from TypeScript's --showConfig. Production packaging is unverified; configuration was not changed to hide these failures.
- Available Bun runtime is 1.3.14, while the unchanged packageManager pin is 1.4.2. Validation results describe the available runtime, not a verified run with the pinned version.
- Browser tooling has no enabled browser; in-app browser creation reports unavailable. Persian/English theme screenshots, actual focus/portal geometry, upload dialogs, permission flows, and replay playback were not browser-verified. No CI or deployed proof is claimed.
- Existing README.md changes remain user-owned and untouched by this migration.

## Release validation follow-up (2026-10-05)

For the requested split commits and web release, repeated formatting, lint, strict typecheck and all
65 Vitest files / 197 tests; all passed. The CI-equivalent frontend Dockerfile built successfully
with pinned Bun 1.4.2, frozen lockfile and the production public API URL. Started the resulting
runtime image in a temporary container; GET /api/health returned status ok and service mahfel-web.
The temporary container was stopped and removed. This resolves the production-packaging gap above
without changing Next configuration; earlier host build failures remain recorded as historical
validation attempts. Browser/UI, CI and deployment proof are separate from this local image check.

Invitation copy buttons now copy the current site origin followed by /join?code= and the
URL-encoded invitation code; localized labels and feedback say Copy link. AGENTS.md and DESIGN.md
record this convention alongside shadcn maxing. Release tags version the independently released
web image; package.json is not bumped merely to match an image tag.
