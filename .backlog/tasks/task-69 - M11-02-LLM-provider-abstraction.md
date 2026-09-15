---
id: TASK-69
title: M11-02 LLM provider abstraction
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
labels: []
milestone: m-11
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 77000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Vendor-specific APIs and failures should not shape domain logic.

Implementation scope: OpenAI-compatible/LiteLLM adapter, model capability config, streaming, usage, timeout/retry/cancel and normalized errors.

Dependencies: M8
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Fake plus configured endpoint pass contract; token/cost/error metrics emitted without prompts.
<!-- AC:END -->
