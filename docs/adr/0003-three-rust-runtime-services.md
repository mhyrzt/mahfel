# ADR 0003: Use three Rust runtime services

## Status

Accepted

## Context

Control-plane request handling, realtime SFU media, and asynchronous jobs have materially different
failure, scaling, networking, and resource profiles.

## Decision

Use `mahfel-api`, `mahfel-media`, and `mahfel-worker` executables. Keep capability boundaries as
modules/crates inside them rather than creating many deployable microservices.

## Consequences

Media can receive stable public networking and drain separately; workers can compete and scale with
queue depth; APIs remain stateless. Internal protocols and three service lifecycles add complexity.

## Alternatives considered

A single Rust binary couples failures and scaling. Per-capability auth/chat/files/AI services create
operational overhead without current production evidence.
