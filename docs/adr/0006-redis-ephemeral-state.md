# ADR 0006: Use Redis only for ephemeral distributed state

## Status

Accepted

## Context

Presence, media-node heartbeats, room placement, leases, and cross-replica coordination need fast
shared state but can be reconstructed.

## Decision

Use Redis for ephemeral coordination only. Durable user-visible facts belong in PostgreSQL or S3.

## Consequences

Redis loss may interrupt presence/placement and require rebuilding state, but cannot erase durable
rooms, chat, files, or identity. Failure behavior must be designed per capability.

## Alternatives considered

PostgreSQL-only coordination increases contention for high-churn state. Redis as primary storage was
rejected because it violates the durability boundary.
