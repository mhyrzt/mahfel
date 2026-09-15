---
id: TASK-62
title: M10-01 STT provider contract
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
labels: []
milestone: m-10
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 70000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Providers differ in audio, streaming, timestamps, languages, and errors.

Implementation scope: Provider trait, normalized request/result, credentials, capability discovery, timeout/retry and cost/usage metrics.

Dependencies: M8, M9
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Fake and one real provider pass contract tests; secrets/audio are absent from logs.
<!-- AC:END -->
