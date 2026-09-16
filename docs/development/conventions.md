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
- Bun is the package manager, script runner, and compatible local runtime.
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
