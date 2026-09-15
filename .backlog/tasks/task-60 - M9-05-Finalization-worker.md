---
id: TASK-60
title: M9-05 Finalization worker
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
labels: []
milestone: m-9
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 68000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Raw streams need playable synchronized artifacts.

Implementation scope: Versioned finalize job, mux/container choice, A/V timestamps, retries/idempotency, thumbnail/metadata and cleanup.

Dependencies: M9-04, M8
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Finalization retry is safe; output plays with synchronized tracks and verified checksum.
<!-- AC:END -->
