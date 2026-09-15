---
id: TASK-59
title: M9-04 Capture and upload
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
labels: []
milestone: m-9
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 67000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Media fragments must reach durable storage without relying on container disk.

Implementation scope: Media tap, bounded spool, segment naming/checksums, multipart/direct S3 upload, backpressure and failure metrics.

Dependencies: M9-01, M9-03, M6-02
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Captured segments appear in S3; storage slowdown is bounded and produces explicit failure.
<!-- AC:END -->
