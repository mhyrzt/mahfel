# ADR 0013: Target stateless PaaS-friendly deployments

## Status

Accepted

## Context

Managed container platforms provide container scheduling and horizontal scaling. Container
filesystems and replica identities are not durable.

## Decision

Application containers require no persistent volumes. Durable state lives in PostgreSQL/S3 and
replaceable coordination in Redis/RabbitMQ. Keep platform-specific networking/configuration out of
domain crates.

## Consequences

API and worker replicas scale/roll cleanly. Media nodes still hold live-room state and need stable
public networking plus graceful draining. External managed infrastructure becomes essential.

## Alternatives considered

Per-container persistent application volumes impair scaling and failover. Hardcoded platform
behavior would prevent portable testing and alternative deployments.
