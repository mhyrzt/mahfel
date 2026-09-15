---
id: TASK-1.1
title: Scaffold repository and Rust workspace
status: Done
assignee:
  - '@codex'
created_date: '2026-09-07 23:24'
updated_date: '2026-09-15 02:04'
labels: []
milestone: m-0
dependencies: []
parent_task_id: TASK-1
ordinal: 2000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: no Rust workspace or service boundary exists. Scope: create toolchain, workspace manifests, three binaries, meaningful shared crates, configuration, gRPC protocol generation, operations listeners, dependency checks, and graceful shutdown.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Cargo workspace builds on Rust 1.95 edition 2024
- [x] #2 API exposes only the minimal status route plus private operations endpoints
- [x] #3 Media initializes configured mediasoup workers and serves the internal gRPC boundary
- [x] #4 Worker establishes RabbitMQ connectivity and exposes truthful readiness
- [x] #5 Protobuf generation works without host protoc
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Follow the approved Mahfel bootstrap plan for this focused slice; update progress and validation evidence through the Backlog CLI.
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Created the Rust 1.95 edition-2024 workspace, minimal API/media/worker binaries, layered configuration, vendored-protoc gRPC boundary, native mediasoup runtime, RabbitMQ readiness, private operations listeners, and graceful shutdown. Full workspace fmt/check/clippy/test passed in the Docker validation target.
<!-- SECTION:FINAL_SUMMARY:END -->
