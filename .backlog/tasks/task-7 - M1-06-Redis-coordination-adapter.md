---
id: TASK-7
title: M1-06 Redis coordination adapter
status: Done
assignee:
  - '@mahyar'
created_date: '2026-09-08 00:31'
updated_date: '2026-09-19 16:11'
labels: []
milestone: m-1
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 15000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Replicas need ephemeral shared coordination without making Redis authoritative.

Implementation scope: Async pool, namespaced keys/TTL, pub-sub strategy, failure classification, health metrics, tests.

Dependencies: M1-01
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Ephemeral key expires; outage behavior is documented/tested; no durable model exists only in Redis.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Application: add ErrorKind::Unavailable and a minimal CoordinationStore port (set_with_ttl/get/delete/publish) with a CoordinationError that separates Unavailable from Unexpected; export from ports.
2. Config: add a [redis] section (url, key_prefix, connect_timeout_seconds, command_timeout_seconds) with defaults in default.toml and MAHFEL_REDIS__URL on the mahfel-api compose service.
3. New mahfel-redis crate: client via the redis crate ConnectionManager (multiplexed, auto-reconnecting) plus a dedicated pub/sub connection; keys module for namespaced families + TTL; CoordinationStore impl; typed RedisError mapping connection/timeout to Unavailable; RedisSubscriber stream; bounded-label metrics.
4. Serve wiring: connect at startup, readiness probe redis via spawn_probe (outage flips readiness, process stays live), metric registration.
5. ADR: record Redis pub/sub as the cross-replica signal strategy, with Streams deferred unless at-least-once fanout is required.
6. Tests: unit (key namespacing, error mapping) and integration against a disposable Redis (MAHFEL_TEST_REDIS_URL, testcontainers fallback) for TTL expiry, pub/sub round-trip, and outage classification/recovery.
7. Docs + validation (clippy, tests, boot smoke with Redis up/down, docker compose config).
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented the coordination adapter. Application: ErrorKind::Unavailable and a minimal CoordinationStore port (set_with_ttl/get/delete/publish) with CoordinationError separating Unavailable from Unexpected; ApiError maps Unavailable to a 503 with a generic message. Config: [redis] (url, key_prefix, connect/command timeouts) with defaults and MAHFEL_REDIS__URL in Compose. New mahfel-redis crate uses the redis crate ConnectionManager (multiplexed, auto-reconnect) plus a dedicated pub/sub connection forwarding through a bounded queue; namespaced key/channel families; bounded-label metrics (redis_commands_total{outcome}, redis_pubsub_messages_total{direction}, redis_probe_failures_total). Serve wiring connects at startup and runs a redis readiness probe. ADR 0017 records pub/sub as the ephemeral cross-replica strategy with Streams deferred. Validation: workspace clippy clean; mahfel-api 47 tests; mahfel-redis unit + integration (TTL expiry, pub/sub round-trip, outage->Unavailable) against a disposable Redis via MAHFEL_TEST_REDIS_URL; boot smoke with Redis up (ready), down (503 not_ready, process alive), back (ready), SIGTERM exit 0; just docs strict; docker compose config OK.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Delivered the M1-06 Redis coordination adapter: a CoordinationStore port with an Unavailable classification, a mahfel-redis crate (ConnectionManager + dedicated pub/sub connection, namespaced keys with TTLs, bounded-label metrics), a [redis] config section, and a readiness probe that degrades to not-ready during an outage and recovers without restarting the process. ADR 0017 records the pub/sub strategy; Streams are deferred until at-least-once cross-replica fanout is required. No durable model lives only in Redis.
<!-- SECTION:FINAL_SUMMARY:END -->
