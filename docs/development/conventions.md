# Engineering conventions

## Rust

- Edition 2024, pinned stable toolchain, rustfmt, and Clippy.
- Library crates expose typed `thiserror` errors; executables may add `anyhow` context.
- Avoid `unwrap`, `todo!`, `unimplemented!`, and unsafe code in Mahfel production paths.
- Use Tokio-aware graceful shutdown. Liveness is process-local; readiness reflects critical ability
  to accept useful work.
- SQLx migrations are forward-only reviewed artifacts. Do not hide schema changes in startup code.
- Configuration defaults, optional local TOML, then namespaced `MAHFEL_` environment values.
- Do not invent cryptography. Use Argon2id, secure OS randomness, and maintained OIDC/JWT/session
  libraries when identity work begins.

## TypeScript

- Strict TypeScript, ESLint, App Router, React Server Components by default.
- `next build` is packaging-only; `just frontend typecheck` remains mandatory for TypeScript errors.
- Prettier is the canonical frontend formatter; `just frontend fmt` checks and
  `just frontend fmt-write` applies formatting.
- Frontend source comments are rejected. Only targeted `eslint-disable`, `eslint-disable-line`, or
  `eslint-disable-next-line` directives are permitted, and unused directives fail linting.
- Protobuf contracts are formatted and linted with Buf. Use `just proto fmt-check` for a check,
  `just proto fmt` to apply formatting, and `just proto lint` for schema linting.
- Bun is the package manager, script runner, and compatible local runtime.
- Vitest is the frontend unit-test runner with React Testing Library and jsdom. Storybook uses the
  Next.js framework integration for isolated component development and visual review.
- Product code is organized by feature. Shared components use shadcn/ui primitives; do not add a
  second component system without an ADR-level reason.
- Avoid `any`; document the external boundary if an unavoidable untyped value exists.
- Browser configuration uses `NEXT_PUBLIC_` only for values safe to expose.

## Interfaces and reliability

- REST and WebSocket go browser-to-API; WebRTC media goes browser-to-media; files go browser-to-S3
  after API authorization.
- RabbitMQ publishers use confirms. Consumers acknowledge manually, retry bounded transient errors,
  dead-letter poison messages, and enforce idempotency.
- Redis is never the sole home of durable user-visible data.
- Logs are structured. Metrics use bounded labels. Request and trace IDs provide correlation.

## HTTP contract

Public product APIs live under `/v1` (the API has its own host, so `/api` is not repeated) and
share one contract:

- **Request IDs.** Every response carries `x-request-id`. A client-supplied value is propagated;
  otherwise the API generates one. The header is exposed to browser JS via CORS and is the
  correlation source for error reports.
- **Error envelope.** Errors render as `{"error":{"code","message","details?"}}`. Codes are stable,
  lower-snake-case strings such as `invalid_input`, `unprocessable_entity`, `not_found`,
  `unauthorized`, `forbidden`, `conflict`, and `internal_error`.
- **Never 500 for bad input.** Malformed JSON and invalid query/body values map to `422`/`400` with
  the envelope above. Validation and known domain failures must not surface as `500`.
- **Pagination.** List endpoints accept optional `?page` (default `1`, minimum `1`) and `per_page`
  (default `20`, maximum `100`). Responses use
  `{"data":[...],"pagination":{"page","per_page","total?"}}`. Out-of-range values return `400`.
- **Security headers.** Responses set `x-content-type-options: nosniff`, `x-frame-options: DENY`,
  a strict-origin referrer policy, and a restrictive permissions policy.
- **CORS.** The API allows only the configured browser origin and an explicit method/header set;
  unconfigured origins receive no `Access-Control-Allow-Origin`.
- **Authorization.** Protected routes opt into the `Authenticated` extractor, which yields a
  `Principal`; policy is then checked through the application `Authorizer` seam. Routes deny by
  default — with no configured identity, protected endpoints return `401`. Do not expose a route
  without a deliberate reason.
