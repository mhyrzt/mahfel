# ADR 0017: Use Redis pub/sub for ephemeral cross-replica signals

## Status

Accepted

## Context

API replicas must deliver ephemeral signals to each other — presence, typing, awareness, and
cross-replica WebSocket fan-out. Durable, committed events already flow through PostgreSQL as the
source of truth and RabbitMQ as the durable transport. The coordination backend is Redis
(ADR 0006) and must stay replaceable.

## Decision

- Use Redis pub/sub for ephemeral, fire-and-forget fan-out between replicas.
- Treat pub/sub as non-durable: there is no delivery guarantee and no replay. Subscribers tolerate
  loss, reconnect, and forward messages through bounded queues so a slow consumer cannot grow
  without limit.
- Keep durable truth in PostgreSQL; keep durable job/event delivery in RabbitMQ. If a later
  requirement needs at-least-once cross-replica delivery of committed events (for example M5-03),
  evaluate Redis Streams with consumer groups at that point rather than now.
- Namespace every channel family under the configured prefix.

## Consequences

Cross-replica signalling is simple and low-latency, with no retention or consumer-group management.
Messages published while a subscriber is disconnected are lost, which is acceptable for ephemeral
signals but not for committed events. Every consumer must reconnect and tolerate gaps.

## Alternatives considered

Redis Streams now would add retention and consumer-group machinery the current signals do not need.
Routing all realtime traffic through RabbitMQ would misuse a durable transport for high-churn
ephemeral state. PostgreSQL `LISTEN`/`NOTIFY` ties fan-out to database connections and does not
scale as a general bus.
