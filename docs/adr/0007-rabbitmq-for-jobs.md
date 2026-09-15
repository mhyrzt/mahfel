# ADR 0007: Use RabbitMQ instead of Kafka for asynchronous jobs

## Status

Accepted

## Context

Current asynchronous work is command/job-oriented: media processing, recording finalization,
transcription, AI, exports, indexing, and notifications.

## Decision

Use RabbitMQ AMQP 0-9-1 with publisher confirms, manual acknowledgements, bounded retry,
dead-lettering, versioned schemas, transactional outbox, and idempotent workers.

## Consequences

RabbitMQ directly supports useful job delivery semantics and is operationally familiar. Consumers
must still handle duplicates and poison messages. RabbitMQ is not the business source of truth.

## Alternatives considered

Kafka was rejected because durable event-log replay is not a primary requirement and would add
operational cost. Revisit Kafka or RabbitMQ Streams only for demonstrated large replayable event or
analytics workloads.
