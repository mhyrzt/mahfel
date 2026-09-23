# ADR 0024: Generate the WebSocket contract with asyncapi-rust

## Status

Accepted

## Context

The `/v1/ws` protocol has separate client and server message enums and a shared versioned JSON
envelope. Handwritten protocol documentation can drift when variants or fields change. ADR 0015
keeps WebSocket traffic outside the REST OpenAPI document and leaves its contract to a separate
decision.

## Decision

- Generate an AsyncAPI 3 document from the transport message enums with `asyncapi-rust`.
- Keep the dependency and schema annotations in `mahfel-api`. Domain and application crates remain
  independent of documentation frameworks.
- Include the shared `v` and optional `id` envelope fields in every message payload. Document
  client messages as received by the API and server messages as sent by the API.
- Export the checked-in REST and WebSocket contracts together with `just api-docs-export`. The
  Rust handlers and message enums remain the sources of truth; both JSON artifacts are generated
  output under `specs/`.
- Continue using `utoipa` for REST routes, as decided in ADR 0015.

## Consequences

Changing a WebSocket variant changes its generated schema. The API build gains the
`asyncapi-rust` and `schemars` dependencies. The export step adds the common envelope and
directional operation references because the library generates schemas for the tagged message
enums, while the runtime sends them inside the envelope.
