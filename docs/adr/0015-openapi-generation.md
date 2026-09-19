# ADR 0015: Generate the OpenAPI contract from the API transport with utoipa

## Status

Accepted

## Context

Later routes and the `mahfel-web` client need a stable, discoverable REST contract, and hand-maintained
specifications drift from the handlers they describe. `mahfel-api` is Axum-based and already derives
`serde` on its transport DTOs. `mahfel-domain` and `mahfel-application` must stay independent of
transport frameworks, so schema annotations must not leak inward. The realtime WebSocket gateway is
not REST and is outside OpenAPI's scope.

## Decision

- Produce OpenAPI 3.1 from the code with `utoipa`, confined to the `mahfel-api` transport layer.
  Domain and application crates never depend on `utoipa`.
- Transport DTOs derive `ToSchema` and handlers are annotated with `#[utoipa::path]`. Domain enums
  that appear in a contract are represented by transport-level schema types (or `value_type`
  overrides) rather than deriving schemas on domain types.
- Serve the machine-readable document at `GET /v1/openapi.json` on the product host. Enable the
  interactive documentation UI only outside production (config-gated) or on the private operations
  listener, consistent with ADR 0011.
- A test asserts that every registered `/v1` REST route appears in the generated document, so an
  unannotated route fails CI rather than being silently absent.
- WebSocket endpoints are not documented via OpenAPI; if they are documented later, that is a
  separate AsyncAPI decision.

## Consequences

The contract has a single source of truth and cannot drift for annotated routes, and the generated
document can drive TypeScript client generation for `mahfel-web`. The costs are proc-macro
annotations at the transport boundary, a route-coverage test to prevent silent omissions, and a
transport-level mapping for domain enums. `utoipa` becomes a dependency of `mahfel-api` only.

## Alternatives considered

A checked-in, spec-first `openapi.yaml` is language-agnostic and codegen-friendly but drifts from
handlers and doubles the bookkeeping. `aide` is Axum-native with fewer macros but has a smaller and
less certain ecosystem. Deferring the contract entirely leaves discoverability and client codegen
unaddressed past M1.
