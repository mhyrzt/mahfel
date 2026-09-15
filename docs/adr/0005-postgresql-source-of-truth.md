# ADR 0005: Use PostgreSQL as durable source of truth

## Status

Accepted

## Context

Identity, authorization, sessions, attendance, chat, file metadata, and job state require durable,
transactional relationships.

## Decision

PostgreSQL is authoritative. Use SQLx and reviewed migrations rather than a heavyweight ORM.

## Consequences

Durable mutations gain transactions and constraints. Pool limits, migrations, backups, and query
performance are operational responsibilities.

## Alternatives considered

Redis is not durable authority. Document databases weaken relational constraints for the core
model. A heavyweight ORM adds abstraction not currently justified.
