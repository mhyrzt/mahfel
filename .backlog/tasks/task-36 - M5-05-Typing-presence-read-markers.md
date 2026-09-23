---
id: TASK-36
title: M5-05 Typing/presence/read markers
status: Done
assignee:
  - '@codex'
created_date: '2026-09-08 00:31'
updated_date: '2026-09-23 14:58'
labels: []
milestone: m-5
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 44000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Ephemeral UX and durable reading state need separate storage semantics.

Implementation scope: TTL typing/presence in Redis, aggregate events, durable per-user channel cursor, privacy/scale limits.

Dependencies: M1-06, M5-03
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Ephemeral indicators expire; read marker survives restart; no user IDs appear in metrics.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Keep short-lived room presence and typing aggregates in Redis with TTLs. 2. Expose private monotonic per-user channel read markers in PostgreSQL. 3. Push bounded room activity snapshots over authorized WebSocket subscriptions. 4. Test Redis expiry, read-marker restart behavior, privacy, and authorization.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Redis TTL presence/typing aggregates and private monotonic PostgreSQL read markers. Redis expiration and PostgreSQL restart/authorization tests passed in full integration suite.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented ephemeral room activity and durable caller-private read positions.
<!-- SECTION:FINAL_SUMMARY:END -->
