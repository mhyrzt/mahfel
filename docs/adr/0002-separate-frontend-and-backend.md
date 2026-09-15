# ADR 0002: Separate Next.js frontend from Rust services

## Status

Accepted

## Context

The web UI and Rust services have different release cadence, build tools, scaling, and runtime
concerns.

## Decision

Deploy `mahfel-web` independently. Rust contains no HTML templates, HTMX, Tailwind compilation,
React rendering, or frontend assets.

## Consequences

Browser/API contracts and CORS need deliberate management. Each side builds and scales without
embedding the other's toolchain or assets.

## Alternatives considered

Serving bundled Next/static assets from Axum and server-rendering backend templates would couple
deployments and were rejected.
