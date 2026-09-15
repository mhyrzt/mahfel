# ADR 0010: Use internal gRPC between API and media

## Status

Accepted

## Context

The control plane needs typed synchronous commands and statistics from media nodes without leaking
mediasoup internals into domain modules.

## Decision

Use tonic/prost gRPC. The browser never calls gRPC. Browser WebRTC signaling remains on
`mahfel-api`; the API translates to the private media control contract.

## Consequences

The boundary is generated, versionable, and testable. Compatibility and deadlines need deliberate
handling. Port 50051 must remain private. The initial contract is a skeleton, not a finalized API.

## Alternatives considered

Internal REST loses stronger generated contracts. Direct shared-process calls prevent independent
media deployment. Exposing media gRPC to browsers leaks implementation and authorization concerns.
