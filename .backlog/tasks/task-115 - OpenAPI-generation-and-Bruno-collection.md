---
id: TASK-115
title: OpenAPI generation and Bruno collection
status: In Progress
assignee: []
created_date: '2026-09-19 15:24'
updated_date: '2026-09-19 15:48'
labels: []
dependencies:
  - TASK-8
references:
  - docs/adr/0015-openapi-generation.md
  - docs/adr/0016-bruno-collection.md
documentation:
  - docs/adr/0015-openapi-generation.md
  - docs/adr/0016-bruno-collection.md
priority: medium
ordinal: 123000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement the recorded decisions: code-first OpenAPI 3.1 generated from the mahfel-api transport with utoipa (ADR 0015), and the bruno/ collection generated from that document (ADR 0016). The contract needs one source of truth so routes cannot silently drift and the frontend/manual testing can consume it.

Scope: utoipa confined to mahfel-api; transport DTOs derive ToSchema with transport-level schema types for domain enums; serve GET /v1/openapi.json with the docs UI non-production/operations-only; a test asserting every /v1 REST route appears; then import/sync a Bruno collection covering REST and manual WebSocket requests, run local-first.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 GET /v1/openapi.json serves an OpenAPI 3.1 document generated from transport DTOs and handlers.
- [x] #2 A test fails when a registered /v1 REST route is missing from the document.
- [x] #3 The interactive docs UI is reachable only outside production or on the operations listener.
- [ ] #4 The bruno/ collection is generated from the document and covers /v1/me and the room flow (create, get, list, join, members, archive) plus manual /v1/ws requests.
- [x] #5 Domain and application crates do not depend on utoipa.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add utoipa to mahfel-api; derive ToSchema on transport DTOs; annotate handlers. 2. Assemble the OpenAPI document and serve GET /v1/openapi.json; gate the docs UI to non-production/operations. 3. Add the route-coverage test. 4. Import/sync the bruno/ collection with local and compose environments (REST + WS). 5. Update docs and validate (clippy, tests, docs).
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
OpenAPI implemented with utoipa 5.5 confined to mahfel-api: transport DTOs derive ToSchema, handlers carry #[utoipa::path], and transport-level visibility/role schema enums keep mahfel-domain free of utoipa. GET /v1/openapi.json serves an OpenAPI 3.1 document; the interactive Scalar UI is mounted at /docs on the private operations listener (ADR 0011/0015); tests cover both route coverage (REST_PATH_TEMPLATES) and a served document. Bruno collection deferred at user request: bruno/.gitkeep placeholder, to be generated from the document via Bruno's OpenAPI import (ADR 0016). Validation: workspace clippy -D warnings clean; mahfel-api 47 tests; boot smoke returned openapi 3.1.0 on the product host, /docs 200 on operations and 404 on the product host; just docs strict. Remains In Progress pending AC4.
<!-- SECTION:NOTES:END -->
