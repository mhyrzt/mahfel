---
id: TASK-41
title: M6-04 Authorized download
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
ordinal: 49000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Private objects need short-lived controlled access.

Implementation scope: Permission check, content disposition, presigned GET, cache policy, audit event.

Dependencies: M6-01, M6-02
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Authorized user downloads directly and unauthorized user receives no usable URL.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Check current context access, issue a short-lived attachment GET with no-store caching, and persist a download audit event.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
API 133 tests passed, including file context authorization; RustFS signed GET integration passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added ACL-gated short-lived signed downloads with attachment/no-store policy and PostgreSQL audit. Route ACL and S3 GET tests pass.
<!-- SECTION:FINAL_SUMMARY:END -->
