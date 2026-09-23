---
id: TASK-40
title: M6-03 Presigned upload
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
ordinal: 48000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: APIs should authorize but not proxy file bodies.

Implementation scope: Initiate contract, limits, short expiry, required headers/checksum, completion verification and abandoned cleanup.

Dependencies: M6-01, M6-02
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Browser uploads directly; oversized/wrong checksum cannot become ready metadata.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Authorize context and quota, issue a short-lived checksum-bound PUT, verify bytes in quarantine, promote to a final key, and clean abandoned staging objects.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Design note: PDFs, single images, and ordered image sequences can be presentation assets uploaded directly to S3-compatible storage through the presigned flow. Keep these as durable artifacts rather than media-plane streams; recording will reference the artifact and capture presenter slide-change events under M9.

Browser file client tests passed; domain/application tests cover size, checksum, and rejected content; RustFS transfer test passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added direct signed upload initiation, quarantine verification, promotion, and worker cleanup. Browser transfer, checksum/size gates, and RustFS transfer tests pass.
<!-- SECTION:FINAL_SUMMARY:END -->
