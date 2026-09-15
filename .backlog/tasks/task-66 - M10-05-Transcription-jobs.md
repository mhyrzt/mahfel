---
id: TASK-66
title: M10-05 Transcription jobs
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
labels: []
milestone: m-10
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 74000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Long-running STT requires durable orchestration.

Implementation scope: Extract/normalize audio, submit/poll/webhook strategy, retries/idempotency, progress, cancellation and DLQ.

Dependencies: M8, M9-05, M10-01
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Recorded meeting produces timestamped persisted transcript after retryable failure.
<!-- AC:END -->
