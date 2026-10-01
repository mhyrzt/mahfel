# ADR 0030: Native recording image rendering in the web application

## Status

Accepted. Supersedes the browser renderer, worker packaging and Next.js independence portions of
[ADR 0028](0028-recording-mp4-export.md). Its durable lifecycle and media policy remain valid.

## Context

The standalone renderer packaged Chromium, Playwright, Bun and frontend code in the worker. The
approved replacement removes the browser and returns PNG, JPEG or self-contained SVG from the
existing web application, on its existing port, protected by a dedicated API key.

## Decision

`mahfel-web` owns `POST /api/internal/recording-render`, a native Next.js route accepting version-one
bounded drawing data. `NextRequest`/`NextResponse` form the HTTP boundary. Authentication, request
parsing, capacity control, validation and drawing are separate functions. The Next.js `nodejs`
runtime exposes native APIs while the container executes Next.js with Bun.

Sharp/Pango renders images and shaped Persian text. RoughJS and perfect-freehand generate drawing
paths. Poppler renders archived PDF pages. SVG responses embed raster text and assets, without
scripts or external references. Native export typography and drawing can differ from interactive
Excalidraw/Lexical browser display; browser pixel parity is not promised.

Bearer authentication uses `MAHFEL_RENDER_API_KEY`; absent configuration denies requests. Only the
exact renderer path bypasses browser session authentication. Archived assets require SHA-256
verification and an explicit storage-origin allowlist. Redirects, arbitrary HTML and arbitrary SVG
input are prohibited. Requests, drawing work, output dimensions, asset bytes, concurrency and
execution time are bounded.

The worker reconstructs Yjs documents through Yrs and owns deterministic recorded-clock replay,
scene policy and GStreamer media composition. It requests PNG overlays over HTTP, caching unchanged
drawing requests. Source preparation generates five-minute internal storage URLs and refreshes them
every four minutes. Renderer failures preserve originals and use the existing fenced export retry
lifecycle. The production worker contains no browser, Bun or frontend bundle.

## Consequences

No new deployable service or listener is introduced. Web availability and rendering capacity become
dependencies of MP4 exports. Web containers require Poppler, fonts and Sharp. CPU work shares the
web process, with two concurrent renders per process; busy requests return a retryable response.
Production web and worker need matching keys, a reachable renderer URL and the web storage
allowlist. Chromium-specific seccomp profiles and shared memory mounts are obsolete. Browser E2E
tooling remains independent under the repository-level `e2e/` directory.

See [the endpoint and export operations contract](../development/recording-export.md).
