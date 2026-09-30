# Mahfel Design System

Mahfel is a realtime communication and collaboration platform for meetings, conversations, classes,
workshops, presentations, and informal gatherings.

The interface should feel calm, modern, human, and focused.

Mahfel is not an "education UI" and should not visually resemble enterprise dashboard software.
Educational use is one use case among many.

The visual system should work equally well for casual team meetings, organization meetings, classes
and lectures, workshops, interviews, presentations, community gatherings, and private conversations.

> **Status.** This document describes the design system as implemented in `frontend/`. Sections
> marked **Future direction** are intended but not built yet; do not treat them as current behavior.
> When this document and the code disagree, the code is the current truth and this document should
> be updated.

---

## 1. Design Principles

### Calm by default

The interface should stay visually quiet unless something requires attention.

Avoid:

- excessive borders;
- excessive shadows;
- large areas of brand purple;
- unnecessary gradients;
- decorative animation during sessions;
- multiple competing accent colors.

Session content should remain the visual focus.

### Color communicates meaning

Do not color controls merely for decoration.

Use color to communicate:

- active state;
- selected state;
- destructive action;
- connection state;
- warning;
- error;
- user attention.

Most interface elements should remain neutral.

### Content first

During a session, the hierarchy is:

1. people/content being presented;
2. active conversation;
3. critical controls;
4. contextual information;
5. navigation.

Chrome should disappear into the background.

### Progressive disclosure

Do not expose every action at once.

Common actions stay visible. Less common actions belong in contextual menus, popovers, command
menus, or secondary panels.

Visible session controls today: microphone, camera, screen share, participants, notes, whiteboard,
presentation, and leave. Secondary actions (moderation, device settings, invite management,
recording, statistics) live in contextual menus and panels.

### Familiar, but distinctly Mahfel

Users already understand products such as Meet, Zoom, Teams, Discord, Slack, and Telegram.

Do not reinvent established interaction patterns without a strong reason. Mahfel's identity should
come from typography, spacing, motion, Persian-aware layout, iconography, color, the room/session
model, and subtle brand details, not unusual UX for its own sake.

---

## 2. Product Vocabulary

Use neutral collaboration vocabulary.

Preferred concepts:

- Organization
- Space
- Room
- Session
- Participant
- Host
- Moderator
- Member
- Guest
- Conversation
- Chat
- Screen share
- Recording
- Notes
- Files

Avoid globally encoding educational terminology such as classroom, teacher, student, or lesson. Those
concepts may exist inside education-specific configurations.

---

## 3. Brand Personality

Mahfel should feel warm, composed, modern, trustworthy, conversational, technically competent, and
culturally aware.

Mahfel should not feel corporate-heavy, childish, futuristic for its own sake, overly playful,
academic, sterile, or aggressively "AI".

---

## 4. Color System

Mahfel uses a neutral UI with an indigo-violet brand accent.

Brand purple should generally occupy a small portion of any screen. Target visual balance:

- ~80% neutral surfaces
- ~15% brand/accent
- ~5% semantic states

The primary brand color is `#6952E8` in light mode and `#8068F2` in dark mode.

The implementation deliberately does **not** ship a separate raw brand scale. Everything is expressed
through the semantic tokens below. Add a raw scale only when a concrete need (charts, illustrations,
brand assets) requires it.

Never reference raw colors directly inside components when a semantic token exists.

### Light

Defined in `app/globals.css` under `:root`:

```css
--background: #fafafc;
--foreground: #181725;
--card: #ffffff;
--card-foreground: #181725;
--popover: #ffffff;
--popover-foreground: #181725;
--primary: #6952e8;
--primary-foreground: #ffffff;
--secondary: #f4f3f8;
--secondary-foreground: #252431;
--muted: #f4f3f8;
--muted-foreground: #706e7c;
--accent: #efecff;
--accent-foreground: #43349d;
--success: #2f9e6f;
--success-foreground: #ffffff;
--warning: #d99022;
--warning-foreground: #181725;
--destructive: #e5484d;
--destructive-foreground: #ffffff;
--border: #e4e2ea;
--input: #e4e2ea;
--ring: #8068f2;
```

### Dark

Defined under `:root[data-theme="dark"]` (and `:root[data-theme="system"]` under
`prefers-color-scheme: dark`):

```css
--background: #101018;
--foreground: #f4f3f8;
--card: #181824;
--card-foreground: #f4f3f8;
--popover: #1b1b27;
--popover-foreground: #f4f3f8;
--primary: #8068f2;
--primary-foreground: #ffffff;
--secondary: #20202d;
--secondary-foreground: #ecebf2;
--muted: #20202d;
--muted-foreground: #a5a3b0;
--accent: #292345;
--accent-foreground: #dcd5fc;
--success: #46c98c;
--success-foreground: #101018;
--warning: #f2b84b;
--warning-foreground: #101018;
--destructive: #ff6369;
--destructive-foreground: #ffffff;
--border: #30303f;
--input: #30303f;
--ring: #a997f5;
```

---

## 5. Themes

Mahfel supports `system`, `light`, and `dark`.

- `system` is the default and follows the operating system preference until the user chooses a mode.
- The choice is persisted in the `mahfel_theme` cookie and applied by the server before first paint,
  so there is no theme flash.
- The theme is expressed as `data-theme` on the `<html>` element. It is not a `.dark` class.
- Components must not depend on a particular theme or use raw colors when a semantic token exists.

Do not assume dark mode is preferred for sessions. The application surface follows the user's
selected appearance; only the session control chrome and video-adjacent surfaces are intentionally
dark (see Section 47).

Both themes must be treated as first-class and checked for contrast, focus visibility, and readable
controls.

---

## 6. Tailwind

Tailwind v4 is configured CSS-first in `app/globals.css` with `@import "tailwindcss"` and a
`@theme inline` block that maps semantic tokens to utilities.

Prefer semantic tokens.

Good:

```tsx
<Button className="bg-primary text-primary-foreground">Join room</Button>
```

Bad:

```tsx
<Button className="bg-[#6952E8] text-white">Join room</Button>
```

Good:

```tsx
<div className="border-border bg-card text-card-foreground" />
```

Bad:

```tsx
<div className="border-gray-200 bg-white text-gray-900" />
```

Raw palette values are acceptable only inside:

- illustrations;
- brand assets;
- charts;
- visualization systems;
- exceptional branded experiences.

---

## 7. shadcn/ui

Use shadcn as infrastructure, not as Mahfel's visual identity.

The project uses the `new-york` style with `neutral` base color and `lucide` icons
(`components.json`). Primitives live in `components/ui`.

Do not accept component defaults blindly. Every component should respect Mahfel radius, spacing,
colors, RTL behavior, focus behavior, motion, and density.

Prefer composition over introducing custom primitives when a shadcn/Radix primitive already solves
accessibility correctly.

Primitives currently present include Button, Dialog, DropdownMenu, Popover, Tooltip, Sheet, Tabs,
ScrollArea, Select, Avatar, Separator, Sidebar, Skeleton, SegmentedControl, Badge, Card, Input,
Textarea, and message/bubble/attachment components used by chat and files.

Do not add a second component library (ADR 0014).

---

## 8. Border Radius

Mahfel should feel soft but not bubbly.

The base radius token is:

```css
--radius: 0.5rem;
```

Typical use in the current implementation:

- buttons and inputs: `rounded-md` (8px);
- cards: `rounded-xl` (12px);
- session control bar: `rounded-2xl`;
- dialogs, sheets, and menus: `rounded-lg`–`rounded-xl`;
- pills, avatars, and status dots: `rounded-full`.

Avoid excessive pill-shaped UI.

---

## 9. Spacing

Use Tailwind's spacing scale.

Primary rhythm: 4, 8, 12, 16, 20, 24, 32, 40, 48, 64.

Use tighter spacing for controls and denser spacing inside sessions. Use more whitespace in
workspace, room discovery, settings, and onboarding surfaces.

---

## 10. Typography

Mahfel must support Persian and English equally well. Do not treat Persian as translated English.
Typography must be tested independently in RTL.

The application loads Vazirmatn (variable) from `@fontsource-variable/vazirmatn` and uses:

```css
font-family: "Vazirmatn Variable", "Inter", system-ui, sans-serif;
```

If Vazirmatn is unavailable, choose another high-quality Persian UI font. Avoid mixing visibly
incompatible Persian and Latin fonts.

### Type Scale

```text
12px — metadata
13px — compact labels
14px — body small
16px — normal body
18px — emphasized body
20px — section title
24px — page title
30px — major heading
36px+ — marketing only
```

Use weight rather than size excessively. Typical weights:

- 400 normal
- 500 UI emphasis
- 600 titles
- 700 rarely

Avoid `font-bold` everywhere.

---

## 11. Persian and RTL

RTL support is a first-class requirement.

Do not implement RTL as a final translation pass.

Locale is resolved from the `NEXT_LOCALE` cookie on the server and applied as `lang` and `dir` on
`<html>`. There are no locale route segments.

Use logical CSS properties wherever possible.

Prefer:

```css
margin-inline-start
margin-inline-end
padding-inline
inset-inline-start
border-inline-start
```

Avoid:

```css
margin-left
margin-right
padding-left
padding-right
```

unless physical direction is truly intentional.

Tailwind should prefer directional-safe utilities (`start-*`, `end-*`, `ms-*`, `me-*`, `ps-*`,
`pe-*`) and `rtl:` / `ltr:` variants for direction-specific adjustments.

---

## 12. Directionality Rules

Some elements mirror in RTL: back/forward navigation, chevrons, breadcrumbs, sidebar placement, and
directional navigation.

Some elements do not mirror: play, microphone, camera, volume, download, upload, checkmark, power,
and media controls with established meaning.

Do not mechanically mirror every SVG. Apply mirrored rotation explicitly where needed (for example
`rtl:-rotate-90` on the landing-page arrow).

---

## 13. Numbers

Persian interfaces may contain Latin numbers, Persian numbers, phone numbers, timestamps, IDs, URLs,
code, and technical metrics.

Do not force Persian digits into technical identifiers. Room IDs, `1080p`, `24 fps`, `42 ms`, codec
names, and similar technical values should remain technically readable.

---

## 14. Icon System

Icons come from `lucide-react` (the configured shadcn icon library). This keeps icon geometry
consistent with the rest of the component system and avoids a bespoke set that drifts.

Guidelines:

- pass `strokeWidth` around `1.8` for standard controls, matching the session toolbar;
- use `aria-hidden` on decorative icons and an accessible label on icon-only controls;
- inherit color with `currentColor` (lucide default) — never hardcode Mahfel purple in an icon.

Primary sizes:

```text
16px (size-4)  — metadata and inline icons
20px (size-5)  — standard UI and navigation
24px (size-6)  — prominent actions and states
matching control size — session controls
```

**Future direction.** A purpose-built Mahfel icon set (24 × 24 canvas, 1.75px stroke, round
caps/joins) is not implemented. Do not start one without an explicit design decision; use lucide
until then.

---

## 15. Buttons

Buttons use `class-variance-authority` with the following variants and sizes.

Variants:

- `default` — `bg-primary text-primary-foreground`, for the dominant action;
- `secondary` — `bg-secondary text-secondary-foreground`, for supporting actions;
- `outline` — bordered neutral action;
- `ghost` — transparent, hover `accent`; preferred for toolbars;
- `destructive` — destructive actions only;
- `link` — inline text action.

Sizes: `default` (h-10), `sm` (h-9), `lg` (h-11), `icon` (40 × 40).

Do not place multiple primary buttons beside each other unless absolutely necessary.

Do not use red for generic disabled states.

---

## 16. Session Controls

Session controls are a special interaction environment. They must remain obvious, reachable,
high-contrast, consistent, and difficult to trigger accidentally.

The control bar is intentionally dark regardless of the application theme: `rounded-2xl`,
`border-slate-800`, `bg-slate-950`, with a soft drop shadow.

Current order:

```text
Mic
Camera
Screen share
──────
Participants
Notes
Whiteboard
Presentation
──────
Leave
(+ host end-session action when present)
```

Controls are `size-10` icon buttons in the `ghost` variant:

- inactive: `text-slate-400`, hover `bg-slate-800 text-white`;
- active/pressed: `bg-slate-800 text-white`;
- leave: `bg-rose-500 text-white`, hover `bg-rose-400`.

Each control has an accessible label and a tooltip. The exact responsive layout may change by
viewport.

**Future direction.** Meeting control targets are currently 40 × 40. A 44–48px minimum for session
controls is a target, not current behavior.

---

## 17. Session Control States

- Microphone on / camera on: the toolbar shows the active (pressed) slate treatment.
- Microphone off / camera off on the toolbar: neutral inactive state. Muting yourself is not an
  error, so the self-toggle is not red.
- Muted participants are marked at the roster and tile level with a `destructive` badge, not only by
  color.
- Screen sharing: represented by the pressed control state and a dedicated screen-share tile.
- Recording: must use a clear, persistent, textual/iconographic indicator. Recording status must not
  depend on color alone.

**Divergence note.** An earlier draft of this document specified a red self-mute toggle and a primary
purple active-screen-share state. The implementation uses the neutral slate pressed treatment. If a
red mute toggle or purple share state is wanted, change the code and this section together.

---

## 18. Leave vs End Session

These are not the same action.

Participant: `Leave`.

Host: `Leave` plus `End for everyone`.

"End for everyone" requires confirmation. Never place destructive confirmation as the default focused
action.

---

## 19. Video Tiles

Video tiles should prioritize people and avoid heavy framing.

Current implementation:

```text
border-radius: 12px (rounded-xl)
background: slate-950
ring: 1px white/10
```

Overlays are minimal: participant name, mute state, speaking state, moderation actions. Keep overlays
out of the way of content.

---

## 20. Active Speaker

Do not use large glowing neon borders.

The implementation marks the active speaker with a `success`-colored ring and a subtle pulsing
wave around the avatar, and shows a speaking indicator. The animation is disabled under reduced
motion.

**Divergence note.** An earlier draft specified a primary purple active-speaker border. The
implementation uses the semantic `success` color. Treat `success` as the current speaking color and
keep it subtle.

---

## 21. Avatar Fallback

When video is disabled, show avatar, name, and audio state.

The current fallback uses a fixed dark slate background with the participant's initials. It does not
yet derive deterministic per-entity colors.

**Future direction.** Deterministic avatar colors that stay stable across sessions are a target.
Until then, do not introduce random per-render colors.

---

## 22. Participant Grid

The stage optimizes for participant count, available space, active surface, and screen share.

Implemented layouts and surfaces are driven by the participant count and the selected surface:

- **Participants** — a responsive grid that sizes tiles and avatars from CSS variables computed for
  the viewport and row/column count;
- **Presentation** — shared presentation content;
- **Notes** — collaborative notes;
- **Whiteboard** — collaborative whiteboard.

Do not expose unnecessary layout complexity to casual users. Sidebar/speaker/picture-in-picture
layout switching is not implemented yet.

---

## 23. Screen Sharing

Screen sharing is content-first.

Shared content receives the largest available area; participant videos become secondary. The
presenter sees an obvious indication that sharing is active.

The screen-share tile and placeholder use a dark, content-neutral treatment.

---

## 24. Chat

Chat should feel lightweight: compact messages, subtle grouping, timestamps when useful, and
reactions without excessive decoration.

Avoid oversized chat bubbles. On desktop, chat and conversation tools live in a side panel rather
than covering content.

---

## 25. Side Panels

Chat, participants, details, and similar tools use the shared panel system.

The session sidebar is `22rem` (352px) on desktop and bound to the logical start side, which mirrors
to the right in RTL. On mobile, panels present as full-screen sheets.

Only one major meeting side panel should normally be open at once.

---

## 26. Notifications

Use three levels:

- **Transient notice** — short confirmation attached to the current view (for example the in-session
  chat notice). There is no global toast library; a transient notice is a small fixed card with
  border, shadow, and a dismiss control.
- **Inline status** — state attached to a component (for example "Connecting…").
- **Dialog** — requires action or contains significant consequences (for example "End session for
  everyone?").

Do not use dialogs for trivial information.

**Future direction.** A general-purpose toast system is not implemented. Add one only if a concrete
need appears beyond the existing transient notice pattern.

---

## 27. Connection States

Mahfel is a realtime system, so connection state is part of the design system.

The signaling client exposes: `connecting`, `connected`, `reconnecting`, `unauthorized`,
`unavailable`, `closed`.

Do not expose raw WebRTC terminology to normal users.

Bad: `ICE connection failed`.
Better: `Connection lost. Reconnecting…`.

Advanced diagnostics may expose technical details separately.

---

## 28. Latency and Quality

**Future direction.** Quality and latency indicators are not implemented.

When they are added, normal users should see only Good / Unstable / Poor, while advanced diagnostics
may expose RTT, jitter, packet loss, bitrate, codec, resolution, and FPS.

---

## 29. Empty States

Empty states should be concise, for example:

```text
No rooms yet.

Create a room to start a conversation.
```

Avoid large decorative illustrations for every empty state.

---

## 30. Loading

Prefer skeletons when the layout is known; the workspace and route loading UI use skeletons and
route-level `loading.tsx` boundaries.

Use a spinner only when the action duration is unknown, a skeleton is inappropriate, or control-level
progress is needed.

Never block the whole UI unnecessarily.

---

## 31. Motion

Motion should communicate state.

Recommended duration:

```text
100–150ms — hover / press
150–200ms — menus / popovers
200–250ms — dialogs / sheets
250–350ms — larger transitions
```

Implemented session animation uses a `cubic-bezier(0.2, 0.8, 0.2, 1)` easing for stage and
participant entry (`220–380ms`) and a slow `speaker-wave` pulse for the active speaker.

Avoid springy animation for core session controls.

---

## 32. Reduced Motion

Always respect `prefers-reduced-motion: reduce`.

Disable decorative movement, unnecessary transforms, and continuous ambient animation. Preserve
state changes.

The session animations are already gated behind `prefers-reduced-motion: no-preference`.

---

## 33. Accessibility

Target WCAG AA minimum.

All controls require keyboard access, visible focus state, an accessible label, appropriate ARIA
semantics, and sufficient contrast.

Icon-only buttons must include an accessible name.

```tsx
<Button variant="ghost" size="icon" aria-label="Mute microphone">
  <MicOff />
</Button>
```

Use the Storybook accessibility addon for reusable components.

---

## 34. Tooltips

All unfamiliar icon-only actions should have tooltips. Session controls expose labels such as mute
microphone, turn camera off, share screen, participants, notes, whiteboard, presentation, and leave.

Tooltips supplement accessible labels; they do not replace them.

---

## 35. Focus States

Use visible focus rings: a 2px `--ring` ring with a 2px offset, already applied through the shared
button styles.

Do not remove browser focus behavior without replacing it.

---

## 36. Hover Is Optional

Every interaction must work without hover. This matters for touch, mobile, tablets, accessibility,
and hybrid devices.

Do not hide essential controls exclusively behind hover. Where actions appear on hover, also reveal
them on focus and keep them permanently visible on non-hover devices.

---

## 37. Responsive Layout

Primary targets: mobile, tablet, desktop, large desktop.

Session UX should not simply shrink the desktop layout.

Mobile priorities:

1. current speaker/content;
2. microphone;
3. camera;
4. leave;
5. essential contextual controls.

Secondary actions move into sheets/menus.

---

## 38. Desktop Navigation

The main application uses a stable, narrow sidebar rail on desktop and a mobile navigation bar on
small screens.

The rail is `w-16`, contains the Mahfel mark, navigation for Rooms and Sessions, a contextual session
action area, the theme control, and the account menu. Navigation stays shallow; room and session
browsing happens in dialogs rather than deep nested routes.

Do not deepen navigation without a clear reason.

---

## 39. Command Palette

**Future direction.** A command palette is not implemented. If added, it should cover actions such as
create room, join room, search rooms, open settings, mute microphone, toggle camera, and open chat.
Keyboard-first workflows are encouraged on desktop.

---

## 40. Keyboard Shortcuts

**Future direction.** Global session shortcuts (for example M for microphone, V for camera, C for
chat, P for participants) are not implemented. When added, avoid overriding common browser/system
shortcuts and make them discoverable.

---

## 41. Cards

Cards are for meaningful grouping, not every piece of content. Avoid "card soup".

Good candidates: room preview, upcoming session, settings group.

Bad candidates: every row, every field, every navigation item.

Cards use `rounded-xl border bg-card text-card-foreground` with a subtle shadow.

---

## 42. Shadows and Elevation

Use very subtle shadows. Default surfaces rely on contrast, border, and spacing rather than shadows.

Elevation is reserved for dialogs, popovers, menus, and the floating session control bar. Avoid large
blurred SaaS-style shadows.

---

## 43. Gradients

Brand gradients are reserved for marketing, onboarding, logo treatment, and rare premium brand
moments.

Inside the working application, gradients appear only as narrow functional treatments (for example
the landing-page radial brand glow, the screen-share placeholder, and the audio level meter). Session
controls remain flat.

---

## 44. Brand Presence

Mahfel should be recognizable without placing the logo everywhere. Brand identity comes from color,
icons, typography, radius, spacing, interaction, motion, and RTL quality.

The logo appears in the shell rail, the landing header, and the app mark, not on every surface.

---

## 45. Persianized Design

"Persianized" does not mean adding traditional ornamentation to every component.

Use Persian influence subtly through icon geometry, typography, language-aware spacing, motion, logo
curves, illustration style, and culturally appropriate visual balance.

Avoid ornamental borders in functional UI, fake calligraphy, excessive traditional motifs, and
stereotypical Persian decoration. Decorative Persian visual language belongs in campaigns,
illustrations, onboarding, landing pages, and special events, not core controls.

---

## 46. Room Cards

A room card may include room name, organization/space, participant presence, next scheduled session,
privacy state, and a primary action.

Avoid displaying every available metadata field.

---

## 47. Session Background

Session control chrome and video-adjacent surfaces are intentionally dark (`slate-950`/`slate-900`)
so video tiles read well and controls stay high-contrast, independent of the user's application
theme.

The session workspace itself follows the application theme. Do not force the entire application dark
for sessions.

**Divergence note.** An earlier draft fixed the whole session background to `#101018`. The
implementation keeps a dark control bar and video surfaces within a theme-aware workspace.

---

## 48. Recording and Privacy

Recording state must be unmistakable and persistent, and must not rely on color alone.

Before recording begins, participants should receive clear notice according to product policy, using
the server's authoritative recording state rather than a locally toggled control.

Never make recording visually subtle.

---

## 49. Presence

**Future direction.** Individual presence states (online, away, busy, offline, in session) are not
implemented as a general feature. When added, keep them subtle, use dot indicators sparingly, and
never rely on color alone — expose text in detailed contexts.

---

## 50. Status Language

Prefer human language.

Bad: `RTC transport disconnected`. Better: `Connection interrupted`.

Bad: `Producer paused`. Better: `Camera paused`.

Technical terminology belongs in diagnostics.

---

## 51. Error Messages

Errors should contain what happened, whether the user needs to act, and a recovery action when
possible.

Example:

```text
We couldn't access your microphone.

Check your browser permissions and try again.
```

Avoid raw error identifiers such as `PermissionDeniedError`.

---

## 52. Dangerous Actions

Require confirmation for actions with broad impact: delete room, end session for everyone, remove
organization, revoke recording, permanently delete account.

Do not confirm routine reversible actions.

---

## 53. Forms

Labels remain visible. Do not use placeholders as labels.

Good:

```text
Room name
[ Weekly engineering sync ]
```

Bad:

```text
[ Enter room name... ]
```

Use descriptions only where ambiguity exists.

---

## 54. Settings

Settings are grouped by domain, for example:

```text
Profile
Audio & Video
Language
Appearance
Calendar
```

Advanced technical settings should not clutter normal preferences. Account settings are reachable
from the rail and open as a dialog on desktop.

---

## 55. Dates and Times

Display according to user locale and the user's calendar preference (Gregorian or Persian/Jalali).

Use `Intl` APIs whenever possible. Do not embed calendar assumptions inside reusable components.

---

## 56. Component Ownership

Reusable, domain-agnostic UI primitives belong in:

```text
components/ui/
```

Mahfel domain components belong inside the feature that owns the behavior:

```text
features/
  sessions/components/
    session-toolbar.tsx
    participant-tile.tsx
    session-stage.tsx
  rooms/
    room-detail.tsx
  shell/components/
    app-shell.tsx
    sidebar-navigation.tsx
```

There is no `components/mahfel/` layer. Do not create one without an architectural decision. Do not
modify low-level primitives for one-off feature behavior.

---

## 57. Class Composition

Use `cn()` from `lib/utils` for conditional Tailwind composition.

```tsx
cn("rounded-lg border bg-card", active && "border-primary", disabled && "opacity-50");
```

Avoid manually concatenating large class strings.

---

## 58. Variants

Use `class-variance-authority` for reusable components with meaningful variants, for example the
button variants and sizes in Section 15.

Do not create variants for arbitrary stylistic differences.

---

## 59. Icon Buttons

Minimum interactive target:

```text
40 × 40px
```

Session controls currently use 40 × 40 and share the button `icon` size. The visible icon may remain
20–24px; touch target matters more than icon size.

**Future direction.** A 44–48px session control target is a goal once the toolbar layout is
revisited.

---

## 60. Z-Index

Maintain a deliberate layer scale.

```text
0  page
10 sticky elements
20 overlays
30 dropdown
40 popover
50 modal
60 toast
70 critical system overlay
```

Avoid arbitrary values such as `z-[999999]`. Current code uses Tailwind/Radix layers in the
`z-10`–`z-50` range; keep new values within the documented scale.

---

## 61. Copy Style

UI copy should be short, direct, conversational, neutral, and useful.

Prefer `Join room` over `Proceed to room`.

Prefer `Microphone is off` over `Your microphone has been deactivated`.

---

## 62. English and Persian Copy

Do not translate mechanically. Persian wording should sound natural in Persian; English wording
should sound natural in English.

Layout should accommodate different text lengths without breaking. Keep copy in the shared message
catalog (`lib/i18n/messages.ts`).

---

## 63. AI Features

**Future direction.** AI features are not implemented.

When they arrive, AI should not dictate Mahfel's core visual identity. AI functionality may use
subtle Mahfel violet, the Parto mascot, a soft glow, or a dedicated surface treatment, but the base
product must remain coherent without AI. Do not place sparkle icons everywhere.

---

## 64. Parto

**Future direction.** Parto, Mahfel's AI companion, is not implemented.

When introduced, use Parto for AI chat, summarization, search assistance, session intelligence,
onboarding/help, and AI-generated actions — not as a generic navigation symbol on every surface.

---

## 65. Design Review Checklist

Before shipping a UI change, verify:

### Visual

- Is the hierarchy obvious?
- Is purple being overused?
- Are spacing and radius consistent?
- Does the screen work in both light and dark themes?

### Interaction

- Is the primary action obvious?
- Are destructive actions clearly separated?
- Are uncommon actions progressively disclosed?

### Accessibility

- Keyboard accessible?
- Focus visible?
- Icon-only actions labeled?
- Contrast acceptable?
- Reduced motion supported?

### RTL

- Tested in Persian?
- Correct reading direction?
- Correct icon mirroring?
- No hardcoded left/right assumptions?

### Responsive

- Works on narrow screens?
- Touch targets large enough?
- Panels and dialogs adapt properly?

### Realtime

- Loading state?
- Reconnecting state?
- Offline/unavailable state?
- Permission-denied state?
- Device-failure state?

---

## 66. Final Rule

When choosing between visual novelty and clarity: **choose clarity.**

When choosing between more UI and less UI: **prefer less UI.**

When choosing whether an interaction should be "Persianized": **preserve familiar behavior and
Persianize the visual language, not the usability model.**

Mahfel should feel like a product built naturally for Persian-speaking users while remaining
completely understandable and comfortable internationally.
