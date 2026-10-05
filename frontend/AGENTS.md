# Mahfel Frontend — Agent Instructions

These instructions apply to everything under `frontend/`.

Also follow the repository-level `../AGENTS.md`.

Mahfel's frontend is an independently deployed web application built on:

- Next.js App Router with React Server Components
- React
- strict TypeScript
- Bun (package manager, script runner, and container runtime)
- Tailwind CSS v4 with CSS-first configuration
- shadcn/ui primitives over Radix UI
- `lucide-react` icons
- Vitest + React Testing Library
- Storybook with the accessibility addon
- Playwright for browser E2E

ADR 0014 defines the stack. Do not introduce a competing component library, CSS framework, state
manager, or form library without an explicit architectural reason. `frontend/DESIGN.md` is the
authoritative visual reference.

## Package Manager and Runtime

Use Bun exclusively for frontend package management and scripts.

Use:

```bash
bun install
bun run ...
```

Do not create npm, Yarn, or pnpm lockfiles.

Respect the Bun version pinned in `package.json`.

## Application Architecture

Organize product code by feature. The current shape is:

```text
frontend/
  app/                      # routes, layouts, route boundaries, composition
  components/
    ui/                     # domain-agnostic shadcn/Radix primitives
  features/                 # product UI and client behavior, grouped by feature
    account/ appearance/ chat/ files/ i18n/ invites/ landing/
    notes/ recordings/ rooms/ sessions/ shell/ whiteboard/ workspace/
  hooks/                    # small shared React hooks
  lib/                      # technical boundaries shared across features
    api/ auth/ collaboration/ i18n/ rtc/ websocket/
  scripts/                  # contract generation, asset preparation and export fixtures
```

Only create a feature directory when a cohesive implementation actually exists.

Do not pre-create empty speculative feature modules.

`features/README.md` is the short orientation map for how the current features fit together. Keep it
current when feature boundaries change.

### `app/`

`app/` owns Next.js routing, layouts, route-level loading/error boundaries, metadata, and
composition.

Do not turn route files into large business-logic modules.

Push feature behavior into the appropriate `features/` module.

`app/globals.css` owns the design tokens and Tailwind theme mapping. It is the single source of truth
for semantic color tokens.

### `features/`

Feature modules own product-specific UI and client behavior.

A feature may contain its own:

- components;
- hooks;
- schemas/types;
- state;
- API adapters;
- tests;
- Storybook stories.

Prefer colocating code that changes together.

Domain components belong inside their feature (for example `features/sessions/components/`). Do not
move feature-specific code into `components/ui` or `lib` merely to make it globally accessible.

Avoid giant global `components`, `hooks`, `types`, `utils`, or `services` directories for
feature-specific code.

### `components/ui/`

Shared, domain-agnostic UI primitives live here. They are generated/adapted shadcn/ui components
built on Radix UI plus locally owned wrappers.

Prefer existing primitives and keep them reusable across features.

Follow **shadcn maxing**: use shared shadcn primitives even for simple buttons, inputs, textareas,
labels, checkboxes, radios, switches, and selects. Compose forms with Field and its label/error
components; use RadioGroup for exclusive choices, Switch for boolean settings, and shared feedback
primitives for errors, empty states, and progress. Product routes/features must pass the ESLint
restrictions on raw ordinary controls.

Keep native file inputs for browser upload plumbing and semantic layout HTML where appropriate.
Retain specialized editor, canvas, calendar, and media behavior; use shared primitives around these
engines. Adapt new registry components to local `cn()`, individual Radix packages, semantic theme
tokens, and RTL direction. Review generated dependencies and diffs; do not overwrite customized
primitives or install the entire component catalog.

Do not introduce another component system merely because a single component is convenient to import.

Keep application-specific styling out of generic `components/ui` primitives.

### `hooks/`

Small hooks shared across more than one feature live here (for example `use-mobile`). Hooks used by
a single feature stay in that feature.

### `lib/`

`lib/` contains technical boundaries shared across features:

- `api/` — typed REST transport, the BFF allowlist, error shape, and generated OpenAPI types;
- `auth/` — NextAuth/Keycloak configuration, callbacks, refresh, and cookie policy;
- `websocket/` — the signaling client and generated message types;
- `rtc/` — mediasoup/WebRTC device, transports, reconnect, and the `useRtc` hook;
- `collaboration/` — the Yjs provider, protocol, and image helpers;
- `i18n/` — locale configuration, message catalog, and server-side locale lookup;
- `utils.ts` — genuinely generic utilities.

Do not move product/domain behavior into `lib/` simply to make it globally accessible.

Generated files (`lib/api/generated/`, `lib/websocket/generated/`) come from the API-owned contract
artifacts. Regenerate them with `bun run generate:api` (or `just frontend generate-api`); never edit
them by hand.

## Server and Client Components

Use React Server Components by default.

Add `"use client"` only when the component actually requires client behavior such as:

- browser APIs;
- local interactive state;
- effects;
- event handlers;
- WebSocket connections;
- media devices;
- WebRTC;
- Yjs/client collaboration;
- client-only libraries.

Keep client boundaries as narrow as practical.

Do not mark an entire route tree as client-side because one leaf component needs browser APIs.

Do not access secrets or server-only configuration from client components.

## Backend Communication

The normal browser architecture is:

```text
Browser ──REST──> mahfel-web BFF ──Bearer(KC JWT)──> mahfel-api
Browser ──WebSocket───────────────────────────────> mahfel-api
Browser ──WebRTC──────────────────────────────────> mahfel-media
Browser ──presigned───────────────────────────────> S3-compatible storage
```

`mahfel-web` authenticates the browser with NextAuth against Keycloak and exposes a narrow BFF for
authenticated REST calls (ADR 0019). The BFF forwards only allowlisted `/v1` paths to the API with
the Keycloak access token and must not become a general proxy. The allowlist lives in
`lib/api/allowlist.ts` and is the boundary that must stay narrow.

Do not proxy WebSocket, WebRTC media, or presigned S3 transfers through Next.js.

Do not widen the BFF beyond the authenticated REST surface or forward arbitrary upstream paths.

ADR 0030 separately permits the API-key authenticated `/api/internal/recording-render` endpoint on
the existing web listener. It produces images from bounded drawing DTOs for worker exports and is
not a proxy. Native rendering lives in `features/recordings/server-render`; route files own HTTP
composition only. Browser E2E tooling lives outside this application, under `../e2e/`.

Never expose access tokens, refresh tokens, or the NextAuth secret to browser JavaScript; the session
is an httpOnly cookie.

Use the configured public API base URL from `lib/api/config.ts` rather than hardcoding
deployment-specific API hosts.

Only expose values through `NEXT_PUBLIC_*` when they are safe to expose to every browser user.

## Authentication

Browser authentication uses NextAuth with the Keycloak provider: a confidential client,
server-side code exchange, and `state` + PKCE (`S256`) + nonce. Keep tokens server-side and the
session cookie httpOnly with secure flags in production (ADR 0019).

Authorization remains authoritative in `mahfel-api`; hiding a control in the UI is not an
authorization decision.

Treat the session and the BFF as security boundaries: validate the upstream path, bound request
size, apply upstream timeouts, and strip cookies and hop-by-hop headers when forwarding.

WebSocket authentication uses a single-use ticket obtained from `POST /v1/ws/tickets` (ADR 0025); do
not pass the raw access token as a long-lived socket credential.

## Invitation Sharing

Invitation copy buttons copy the complete join URL, not the bare code:
`BASE_URL/join?code=CODE`. In browser copy handlers, use `window.location.origin` as `BASE_URL`
and URL-encode the invitation code with `encodeURIComponent`. Use this format for both room and
session invitations. Keep button labels, tooltips, and success feedback localized and worded as
copying a link. The invitation list may still display the bare code.

## API Boundaries

Keep HTTP transport details out of presentation components.

Prefer the typed API boundary in `lib/api` that handles:

- base URL construction;
- request encoding;
- response decoding;
- expected error shape;
- authentication/session transport;
- cancellation where useful.

Use `lib/api/browser.ts` from client code and `lib/api/server.ts` from server code. Do not scatter
raw `fetch()` or `axios` calls across unrelated components.

Do not silently swallow HTTP failures.

UI behavior should distinguish expected states such as:

- loading;
- empty;
- permission denied;
- validation failure;
- unavailable dependency;
- unexpected failure.

Avoid converting every failure into the same generic message.

## WebSockets

WebSocket connections belong in the explicit signaling boundary (`lib/websocket/client.ts`).

Do not create independent socket connections from arbitrary components.

Connection ownership, reconnect policy, subscriptions, and cleanup should be explicit. The current
client uses bounded exponential backoff, a bounded map of pending requests, and explicit status
transitions.

Components should consume typed application events rather than parsing arbitrary transport payloads.
Message types are generated from the AsyncAPI contract (ADR 0024).

Assume reconnects and duplicated or reordered transient events can occur where the protocol permits
them.

Bound client-side queues and retained history.

## WebRTC / mediasoup

Browser media uses `mediasoup-client` through `lib/rtc` (device, transport, reconnect, `useRtc`).

Signaling goes through `mahfel-api`.

Media flows directly between the browser and `mahfel-media`, or through TURN when required (ADR 0004,
ADR 0009).

Session media is consumed by `features/sessions` (`use-session-media`, `use-remote-session-media`,
`remote-audio`). Keep media lifecycle code outside ordinary presentational components.

Explicitly manage:

- permissions;
- devices;
- tracks;
- transports;
- producer/consumer lifecycle;
- disconnect;
- reconnect;
- track replacement;
- cleanup.

Never assume a media track disappears merely because a React component unmounted. Close or stop owned
browser/media resources deliberately.

## Collaboration

Collaborative notes and the whiteboard use Yjs documents over the collaboration gateway (ADR 0012,
ADR 0020). The provider and protocol live in `lib/collaboration`; feature editors (Lexical for notes,
Excalidraw for the whiteboard) build on it.

Keep collaboration transport concerns in `lib/collaboration` and feature-specific editing behavior in
the owning feature.

Treat collaborative content as untrusted input.

## TypeScript

TypeScript is strict.

Do not use `any`.

When consuming genuinely untyped external data, validate or narrow it at the boundary rather than
allowing `any` to spread through the application.

Prefer:

- discriminated unions;
- explicit domain types;
- exhaustive state handling;
- type-only imports;
- narrow component props.

Avoid non-null assertions.

Avoid unsafe casts used only to silence the compiler.

If an external API forces a cast, keep it local to the boundary and make the runtime assumptions
explicit through validation when appropriate.

## Source Comments

Frontend source comments are prohibited by the ESLint configuration (`mahfel/no-comments`) except for
targeted ESLint directives.

Write self-explanatory code instead of explanatory comments.

Permitted directives include forms such as:

```text
eslint-disable
eslint-disable-line
eslint-disable-next-line
```

Use them only when necessary.

Unused disable directives are errors.

Do not add TODO comments. Track unfinished work through Backlog.md.

## React

Prefer component composition over large configurable mega-components.

Keep state as local as possible.

Do not introduce global state management until state genuinely crosses feature/application
boundaries and existing React/Next patterns are insufficient.

Avoid effects for values that can be derived during render.

Effects should synchronize with external systems, not act as a generic state transformation
mechanism.

Clean up subscriptions, timers, sockets, media tracks, and observers.

Use stable keys derived from entity identity rather than array indexes when rendering mutable
collections.

Do not mutate props or shared state objects.

## Styling

Use Tailwind and the existing design-system primitives.

Tailwind is configured CSS-first in `app/globals.css` (`@import "tailwindcss"` plus `@theme inline`).
Prefer semantic token utilities (`bg-card`, `text-muted-foreground`, `border-border`) over raw palette
values such as `bg-gray-200`. Raw values are acceptable only for illustrations, brand assets, charts,
and visualization systems.

Use `cn()` from `lib/utils` for conditional class composition and `class-variance-authority` for
reusable variants.

Avoid one-off inline style objects when Tailwind or an existing primitive expresses the same design.

Do not duplicate a reusable visual primitive across several features.

Keep application-specific styling out of generic `components/ui` primitives.

Preserve responsive behavior.

When adding interactive components, include visible focus behavior and keyboard operation where
applicable.

`frontend/DESIGN.md` documents the intended look, motion, and control behavior; follow it for visual
decisions.

## Internationalization

Locale selection is cookie-based (`NEXT_LOCALE`), not route-segment-based. `lib/i18n/config.ts`
defines the supported locales and direction, `lib/i18n/server.ts` reads the locale on the server, and
`lib/i18n/messages.ts` holds the message catalog. The root layout sets `lang` and `dir` on `<html>`.

Use `messagesFor(locale)` for user-facing text and keep feature copy in the shared catalog rather than
inlining strings.

Preserve RTL correctness and do not hardcode left/right assumptions that break in Persian.

Do not invent a second translation framework, and do not introduce locale route segments without an
architectural decision.

## Appearance and Theme

Appearance is server-rendered from the `mahfel_theme` cookie with values `system`, `light`, or
`dark`. The root layout reads the cookie and sets `data-theme` on `<html>` before paint; dark tokens
are defined under `:root[data-theme="dark"]` and `:root[data-theme="system"]` with
`prefers-color-scheme`.

Persist changes through `features/appearance/theme-preference`.

Do not switch this to a `.dark` class mechanism or a client-only theme toggle that causes a flash of
the wrong theme. Keep both themes first-class and contrast-checked.

## Accessibility

Use semantic HTML before ARIA.

Interactive elements must be keyboard reachable.

Buttons should be buttons; navigation should use links.

Inputs need accessible labels.

Dialogs, menus, tooltips, and similar primitives should use Radix/shadcn primitives rather than
hand-built div-based imitations.

Do not encode important state using color alone.

Use the Storybook accessibility tooling (`@storybook/addon-a11y`) for reusable components.

## Performance

Do not optimize based purely on intuition, but avoid obvious client-side cost.

In particular:

- avoid unnecessarily converting server-renderable trees to client components;
- avoid shipping large dependencies for trivial functionality;
- lazy-load genuinely heavy client-only features when appropriate;
- avoid rerender cascades caused by unnecessarily broad state;
- do not store large binary/media payloads in React state;
- clean up browser media resources;
- paginate or virtualize genuinely large collections when required by measured/product needs.

Do not add memoization everywhere by default.

Use it where referential stability or measured render cost requires it.

## Security

Treat API responses, URL parameters, rich text, filenames, external URLs, and collaborative content
as untrusted.

Do not expose server secrets through `NEXT_PUBLIC_*`.

Do not store sensitive authentication material in browser storage unless the authentication design
explicitly requires it.

Avoid rendering arbitrary HTML.

When rendering rich content becomes necessary, use an explicit sanitization strategy.

Do not bypass authorization UX by hiding controls alone; backend authorization remains authoritative.

## Error Handling

Expected failure states should be represented explicitly in feature state.

Do not use exceptions as ordinary UI control flow when a typed result/state is clearer.

Unexpected errors should reach the appropriate application error boundary or telemetry path.

Error messages shown to users should be useful without exposing backend internals or secrets.

## Testing

Use the smallest test that proves the behavior.

### Unit/component tests

Use Vitest (jsdom) with React Testing Library and `@testing-library/jest-dom` for:

- deterministic utilities;
- hooks;
- state transitions;
- user interactions;
- feature components.

Tests live with the feature under `tests/`.

Test behavior visible to users rather than implementation details.

Prefer role/label/text-based queries over brittle CSS selectors.

### Storybook

Add or update stories for reusable UI components when visual states are meaningful. Stories are
discovered under `components/**` and `features/**`.

Stories should exercise important variants such as:

- normal;
- disabled;
- loading;
- destructive/error;
- relevant responsive or interactive states.

Do not duplicate every feature integration test as a Storybook story.

### Browser E2E

Playwright specs live in the repository-level `../e2e/tests` and target the local stack
(`https://mahfel.localhost` by default).

Do not fake WebSocket or WebRTC behavior when an E2E test's purpose is specifically to verify the
real cross-service boundary.

Full browser E2E belongs with usable cross-service flows rather than trivial component behavior.

## Validation

During development, use the narrowest relevant check.

Before finalizing normal frontend work, run the relevant subset of:

```bash
just frontend fmt
just frontend lint
just frontend typecheck
just frontend test
```

For reusable UI/design-system changes, also consider:

```bash
just frontend storybook-build
```

For changes that affect production packaging or Next.js routing/runtime behavior:

```bash
just frontend build
```

Do not treat `next build` as the TypeScript validation gate. This repository intentionally performs
TypeScript validation separately.

If a command fails because of a pre-existing unrelated issue, distinguish it from regressions caused
by the current change.

## Dependencies

Before adding a frontend dependency:

1. Check whether React, Next.js, the browser platform, Tailwind, Radix/shadcn, or an existing
   dependency already provides the capability.
2. Consider bundle cost.
3. Consider whether it forces a client component boundary.
4. Prefer maintained libraries with narrow responsibilities.

Do not add an entire state, component, form, networking, or utility framework for one small use case,
and do not add a second component library (ADR 0014).

## Before Finishing Frontend Work

Check that:

- server/client boundaries remain intentional;
- feature code lives in the appropriate feature;
- shared UI remains domain-agnostic;
- browser/backend architecture has not been bypassed;
- REST/WebSocket/media/file flows use their intended transports;
- TypeScript has not been weakened;
- resources and subscriptions are cleaned up;
- loading/error/empty states are handled where relevant;
- accessibility remains intact;
- visual changes match `frontend/DESIGN.md` and RTL still works;
- focused tests exist for meaningful behavior;
- relevant frontend gates pass.

<!-- BEGIN:nextjs-agent-rules -->

# This is NOT the Next.js you know

This version has breaking changes — APIs, conventions, and file structure may all differ from your training data. Read the relevant guide in `node_modules/next/dist/docs/` (resolved from this file's directory; in monorepos the `next` package may not be visible from the repo root) before writing any code. Heed deprecation notices.

This block is written and re-added by `next dev` — verify at `node_modules/next/dist/server/lib/generate-agent-files.js`. Removing it from a diff only re-creates the uncommitted change; committing it with your work keeps the tree clean.

<!-- END:nextjs-agent-rules -->
