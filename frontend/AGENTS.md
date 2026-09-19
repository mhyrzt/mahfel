# Mahfel Frontend — Agent Instructions

These instructions apply to everything under `frontend/`.

Also follow the repository-level `../AGENTS.md`.

Mahfel's frontend is an independently deployed web application using:

- Next.js App Router
- React
- strict TypeScript
- Bun
- Tailwind CSS
- shadcn/ui
- Vitest
- React Testing Library
- Storybook

Do not introduce a competing frontend stack without an explicit architectural reason.

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

Organize product code by feature.

Preferred shape:

```text
frontend/
  app/
  components/
    ui/
  features/
    auth/
    classroom/
    meeting/
    chat/
    files/
    notes/
    whiteboard/
    ai/
  lib/
    api/
    websocket/
    rtc/
    collaboration/
```

Only create a feature directory when a cohesive implementation actually exists.

Do not pre-create empty speculative feature modules.

### `app/`

`app/` owns Next.js routing, layouts, route-level loading/error boundaries, metadata, and composition.

Do not turn route files into large business-logic modules.

Push feature behavior into the appropriate `features/` module.

### `features/`

Feature modules own product-specific UI and client behavior.

A feature may contain its own:

- components;
- hooks;
- schemas/types;
- state;
- API adapters;
- tests.

Prefer colocating code that changes together.

Avoid giant global `components`, `hooks`, `types`, `utils`, or `services` directories for feature-specific code.

### `components/ui/`

Shared UI primitives live here.

Prefer existing shadcn/ui primitives and locally owned wrappers.

Do not introduce another component system merely because a single component is convenient to import.

Reusable UI primitives should remain domain-agnostic.

### `lib/`

`lib/` contains technical boundaries shared across features, such as:

- API transport;
- WebSocket transport;
- mediasoup/WebRTC integration;
- Yjs collaboration integration;
- genuinely generic utilities.

Do not move product/domain behavior into `lib/` simply to make it globally accessible.

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
the Keycloak access token and must not become a general proxy.

Next.js is not a general backend-for-frontend proxy for Mahfel.

Do not proxy WebSocket, WebRTC media, or presigned S3 transfers through Next.js.

Do not widen the BFF beyond the authenticated REST surface or forward arbitrary upstream paths.

Never expose access tokens, refresh tokens, or the NextAuth secret to browser JavaScript; the session
is an httpOnly cookie.

Use the configured public API base URL rather than hardcoding deployment-specific API hosts.

Only expose values through `NEXT_PUBLIC_*` when they are safe to expose to every browser user.

## Authentication

Browser authentication uses NextAuth with the Keycloak provider: a confidential client,
server-side code exchange, and `state` + PKCE (`S256`) + nonce. Keep tokens server-side and the
session cookie httpOnly with secure flags in production.

Authorization remains authoritative in `mahfel-api`; hiding a control in the UI is not an
authorization decision.

Treat the session and the BFF as security boundaries: validate the upstream path, bound request
size, apply upstream timeouts, and strip cookies and hop-by-hop headers when forwarding.

## API Boundaries

Keep HTTP transport details out of presentation components.

Prefer a typed API boundary that handles:

- base URL construction;
- request encoding;
- response decoding;
- expected error shape;
- authentication/session transport;
- cancellation where useful.

Do not scatter raw `fetch()` calls across unrelated components.

Do not silently swallow HTTP failures.

UI behavior should distinguish expected states such as:

- loading;
- empty;
- permission denied;
- validation failure;
- unavailable dependency;
- unexpected failure.

Avoid converting every failure into the same generic toast.

## WebSockets

WebSocket connections belong in an explicit transport/client boundary.

Do not create independent socket connections from arbitrary components.

Connection ownership, reconnect policy, subscriptions, and cleanup should be explicit.

Components should consume typed application events rather than parsing arbitrary transport payloads.

Assume reconnects and duplicated or reordered transient events can occur where the protocol permits them.

Bound client-side queues and retained history.

## WebRTC / mediasoup

Browser media uses `mediasoup-client` when the RTC milestone is implemented.

Signaling goes through `mahfel-api`.

Media flows directly between the browser and `mahfel-media`, or through TURN when required.

Keep media lifecycle code outside ordinary presentational components.

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

Never assume a media track disappears merely because a React component unmounted. Close or stop owned browser/media resources deliberately.

## TypeScript

TypeScript is strict.

Do not use `any`.

When consuming genuinely untyped external data, validate or narrow it at the boundary rather than allowing `any` to spread through the application.

Prefer:

- discriminated unions;
- explicit domain types;
- exhaustive state handling;
- type-only imports;
- narrow component props.

Avoid non-null assertions.

Avoid unsafe casts used only to silence the compiler.

If an external API forces a cast, keep it local to the boundary and make the runtime assumptions explicit through validation when appropriate.

## Source Comments

Frontend source comments are intentionally prohibited by the ESLint configuration except for targeted ESLint directives.

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

Do not introduce global state management until state genuinely crosses feature/application boundaries and existing React/Next patterns are insufficient.

Avoid effects for values that can be derived during render.

Effects should synchronize with external systems, not act as a generic state transformation mechanism.

Clean up subscriptions, timers, sockets, media tracks, and observers.

Use stable keys derived from entity identity rather than array indexes when rendering mutable collections.

Do not mutate props or shared state objects.

## Styling

Use Tailwind and the existing design-system primitives.

Avoid one-off inline style objects when Tailwind or an existing primitive expresses the same design.

Do not duplicate a reusable visual primitive across several features.

Keep application-specific styling out of generic `components/ui` primitives.

Preserve responsive behavior.

When adding interactive components, include visible focus behavior and keyboard operation where applicable.

## Accessibility

Use semantic HTML before ARIA.

Interactive elements must be keyboard reachable.

Buttons should be buttons; navigation should use links.

Inputs need accessible labels.

Dialogs, menus, tooltips, and similar primitives should use accessible primitives rather than hand-built div-based imitations.

Do not encode important state using color alone.

Use Storybook accessibility tooling where useful for reusable components.

## Internationalization

Preserve the existing locale-aware routing and message architecture.

Do not hardcode assumptions that make later locale support impossible.

User-facing text that belongs to the localization system should use the project's message mechanism once that feature area is established.

Do not invent a second translation framework.

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

Treat API responses, URL parameters, rich text, filenames, external URLs, and collaborative content as untrusted.

Do not expose server secrets through `NEXT_PUBLIC_*`.

Do not store sensitive authentication material in browser storage unless the authentication design explicitly requires it.

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

Use Vitest and React Testing Library for:

- deterministic utilities;
- hooks;
- state transitions;
- user interactions;
- feature components.

Test behavior visible to users rather than implementation details.

Prefer role/label/text-based queries over brittle CSS selectors.

### Storybook

Add or update stories for reusable UI components when visual states are meaningful.

Stories should exercise important variants such as:

- normal;
- disabled;
- loading;
- destructive/error;
- relevant responsive or interactive states.

Do not duplicate every feature integration test as a Storybook story.

### Browser E2E

Do not fake WebSocket or WebRTC behavior when an E2E test's purpose is specifically to verify the real cross-service boundary.

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

Do not treat `next build` as the TypeScript validation gate. This repository intentionally performs TypeScript validation separately.

If a command fails because of a pre-existing unrelated issue, distinguish it from regressions caused by the current change.

## Dependencies

Before adding a frontend dependency:

1. Check whether React, Next.js, the browser platform, Tailwind, Radix/shadcn, or an existing dependency already provides the capability.
2. Consider bundle cost.
3. Consider whether it forces a client component boundary.
4. Prefer maintained libraries with narrow responsibilities.

Do not add an entire state, component, form, networking, or utility framework for one small use case.

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
- focused tests exist for meaningful behavior;
- relevant frontend gates pass.
