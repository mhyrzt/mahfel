---
id: TASK-39
title: M6-02 S3 adapter
status: To Do
assignee: []
created_date: '2026-09-08 00:31'
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
Problem: Cloud and MinIO endpoints need one tested abstraction.

Implementation scope: AWS SDK client config, endpoint/path-style support, timeouts/retries, bucket checks and error mapping.

Dependencies: M0-07
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Same integration suite passes against MinIO and configured S3-compatible test endpoint.
<!-- AC:END -->
