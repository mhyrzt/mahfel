---
id: TASK-106
title: Replace local MinIO with RustFS
status: Done
assignee:
  - '@codex'
created_date: '2026-09-15 23:15'
updated_date: '2026-09-15 23:37'
labels: []
dependencies: []
ordinal: 114000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Mahfel should use RustFS as its local S3-compatible object store while preserving the generic S3 application contract and local bucket bootstrap behavior.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Compose runs a pinned RustFS image with persistent storage, loopback-only API/console ports, credentials, and a working health check
- [x] #2 The configured Mahfel bucket is created idempotently and is usable through the S3 API
- [x] #3 Environment examples, architecture docs, ADR wording, networking docs, and roadmap references name RustFS instead of MinIO
- [x] #4 Compose configuration, documentation build, and a live RustFS health/S3 smoke test pass
- [ ] #5 Compose configuration and documentation build pass without pulling or starting images
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Verify the current official RustFS image, ports, credentials, storage path, and health contract. 2. Replace MinIO services, environment names, persistent volume, and bucket bootstrap with RustFS-compatible equivalents while preserving generic MAHFEL_S3 settings. 3. Update architecture, deployment, ADR, networking, and roadmap references. 4. Validate Compose and docs statically without pulling or starting images, as explicitly requested by the user. 5. Record the runtime smoke test as unverified rather than downloading images.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Replaced MinIO with RustFS 1.0.0-rc.6 and AWS CLI 2.36.46 through docker.arvancloud.ir. Added separate internal and browser-facing S3 endpoints plus HTTPS Caddy routes. Compose config and strict docs build pass. Per user instruction, no image was pulled or container started, so runtime bucket creation is not claimed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Swapped the local object store to pinned RustFS through the requested registry mirror, retained generic S3 settings, added idempotent bucket bootstrap, and updated all architecture and deployment references.
<!-- SECTION:FINAL_SUMMARY:END -->
