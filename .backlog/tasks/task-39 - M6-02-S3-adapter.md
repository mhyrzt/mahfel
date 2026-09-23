---
id: TASK-39
title: M6-02 S3 adapter
status: Done
assignee:
  - '@codex'
created_date: '2026-09-08 00:31'
updated_date: '2026-09-23 12:34'
labels: []
milestone: m-6
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 47000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Direct file transfers need a tested S3-compatible storage boundary.

Implementation scope: AWS SDK client configuration, endpoint/path-style support, timeouts/retries, bucket checks, and error mapping. RustFS is the required M6 integration target.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The signed transfer integration suite passes against disposable RustFS, including bucket CORS and lifecycle configuration.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Configure bounded AWS S3 clients for internal operations and public presigning; exercise the signed transfer contract against disposable RustFS.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Scope decision (2026-09-23): user confirmed RustFS alone is sufficient for M6; MinIO and external endpoint runs are not required.

RustFS Testcontainer passed signed PUT, bounded read, copy, signed GET, delete, bucket CORS and lifecycle configuration. Local proxy variables were cleared for loopback.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added an AWS SDK S3 adapter with separate internal and public endpoints, bounded reads, retries, and signed transfers. The disposable RustFS contract passed.
<!-- SECTION:FINAL_SUMMARY:END -->
