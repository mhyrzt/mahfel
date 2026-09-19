# ADR 0016: Use a Bruno collection generated from the OpenAPI document

## Status

Accepted

## Context

Developers need to exercise `mahfel-api` interactively and end-to-end, and ad-hoc requests kept
outside the repository drift from the real contract. ADR 0015 makes a `utoipa`-generated OpenAPI
document the single source of truth for REST. Bruno is an offline, Git-friendly API client whose
collections are plain files, it can import an OpenAPI document, and it supports both REST and
WebSocket requests.

## Decision

- Keep a Bruno collection in the repository under `bruno/` as the interactive and end-to-end request
  set for `mahfel-api`.
- Generate the collection from the `utoipa` OpenAPI document. The document remains authoritative, so
  the collection follows it; never export OpenAPI from the collection.
- Cover REST and WebSocket requests. WebSocket requests support manual exercise of the gateway; they
  do not replace its automated tests.
- Run local-first against a local or Compose stack through Bruno environments. A `bru` CLI smoke job
  in CI is a later option, not a requirement now.
- The collection complements, and never replaces, the Rust unit and integration tests.

## Consequences

The contract keeps one source of truth, and contributors get a runnable request set with saved
examples. Regenerating the collection becomes a step when the contract changes, contributors need
Bruno installed, and WebSocket coverage stays manual. The collection is tooling, so it carries no
runtime or deployment dependency.

## Alternatives considered

A hand-maintained collection is simpler to start but becomes a second contract that drifts. A
collection-first workflow that exports OpenAPI would invert ADR 0015. Skipping a collection leaves
interactive and end-to-end exercising to ad-hoc tooling.
