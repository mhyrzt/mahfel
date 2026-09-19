---
id: TASK-112
title: Introduce mahfel-application layer and invert store dependencies
status: Done
assignee: []
created_date: '2026-09-18 15:01'
updated_date: '2026-09-19 15:54'
labels:
  - backend
  - architecture
dependencies: []
priority: high
ordinal: 120000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add the missing application layer between HTTP/gRPC transports and domain so CreateRoom/JoinRoom/StartSession/EndSession/SyncIdentity use cases have a home. Define RoomStore/UserStore/SessionStore/MediaGateway/EventPublisher ports in application, move ID/time creation into domain constructors (Room::create/archive, etc.), change Postgres adapters to insert/save fully-formed entities, and wrap media-client/messaging behind gateway traits. Keeps existing workspace boundary (api/media/worker composition roots, postgres/auth/messaging/rtc adapters).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 mahfel-application crate exists with ports + rooms/users/sessions use cases and depends only on mahfel-domain
- [x] #2 Domain owns Room/User/Session creation invariants and title validation matching migration bounds
- [x] #3 Postgres stores implement application ports via insert/save without generating IDs
- [x] #4 MediaGateway and EventPublisher ports hide tonic/lapin from callers
- [x] #5 cargo clippy, fmt, and workspace tests pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Create mahfel-application crate (ports + rooms/sessions/users use cases, StoreError/MediaGateway/EventPublisher). 2. Enrich domain with RoomTitle/DisplayName constructors + Room::create/archive, Session schedule/start/end/cancel. 3. Rename mahfel-database to mahfel-postgres with Postgres*Store implementing application ports (insert/save). 4. Wrap media-client behind GrpcMediaGateway and messaging behind RabbitMqEventPublisher. 5. Keep bins as composition roots (no handler rewiring yet - leaves TASK-4/5 to consume ports). 6. cargo fmt/clippy/nextest.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Completed hexagon migration: domain invariants (DisplayName/RoomTitle + Room::create/archive, User::create/rename/disable/enable, Session::schedule/start/end/cancel), application ports (RoomStore/UserStore/SessionStore/MediaGateway/EventPublisher) + vertical slices rooms/sessions/users, mahfel-database -> mahfel-postgres with Postgres*Store insert/save, GrpcMediaGateway and RabbitMqEventPublisher hiding tonic/lapin. cargo fmt/clippy pass, lib tests 9 passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Introduced mahfel-application as the hexagon boundary between transports and domain: domain constructor invariants and validation (Room/User/Session, DisplayName/RoomTitle), application ports (RoomStore/UserStore/SessionStore/MediaGateway/EventPublisher) with rooms/users/sessions vertical slices, mahfel-postgres Postgres*Store adapters that insert/save fully-formed entities, and GrpcMediaGateway/RabbitMqEventPublisher hiding tonic/lapin. Workspace fmt/clippy/tests pass. Finalized retroactively; the layer is implemented and consumed by the room endpoints and the WebSocket gateway.
<!-- SECTION:FINAL_SUMMARY:END -->
