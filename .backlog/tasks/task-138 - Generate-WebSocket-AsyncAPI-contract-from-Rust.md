---
id: TASK-138
title: Generate WebSocket AsyncAPI contract from Rust
status: Done
assignee: []
created_date: '2026-09-23 02:15'
updated_date: '2026-09-23 02:43'
labels: []
dependencies: []
ordinal: 146000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Generate an AsyncAPI 3 document from the WebSocket message types using asyncapi-rust, matching the actual v1 envelope and direction of messages. Integrate export with the existing API tooling and document regeneration.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 AsyncAPI artifact covers every client and server WebSocket message and its envelope
- [x] #2 Export is reproducible from the Rust source through a documented command
- [x] #3 WebSocket documentation states authentication, protocol version, and direction accurately
- [x] #4 Frontend generates TypeScript WebSocket message types from specs/asyncapi.json with a documented Bun command
- [x] #5 The WebSocket client consumes generated types without duplicating wire discriminants
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Keep Rust enums as the AsyncAPI source. Export REST and WebSocket contracts together under specs/. Generate TypeScript interfaces from the AsyncAPI message JSON Schemas through Bun, expose client/server unions at the frontend WebSocket boundary, validate generated output and consumer compilation, then commit focused changes.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
The single Clap api-docs export command writes specs/openapi.json and specs/asyncapi.json. Generated AsyncAPI has 16 client and 23 server messages with directional operations. Validated export, backend formatting and lint, frontend generation and typecheck, and strict docs build.

Frontend generation uses json-schema-to-typescript on the AsyncAPI message payloads because Modelina's transitive AJV code generation fails under pinned Bun. The generated 39 interfaces and directional unions are consumed by the RTC WebSocket client. Frontend typecheck, formatting, focused ESLint, and strict docs build passed; full frontend lint reports an existing unused CardHeader import in schedule-panel.tsx.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Generate the 39 WebSocket message schemas from Rust, export both API specs under specs/, generate frontend WebSocket types from AsyncAPI, and use those types for outgoing RTC signaling.
<!-- SECTION:FINAL_SUMMARY:END -->
