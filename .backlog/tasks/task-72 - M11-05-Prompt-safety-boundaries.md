---
id: TASK-72
title: M11-05 Prompt safety boundaries
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
labels: []
milestone: m-11
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 80000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Untrusted classroom content can manipulate tools or disclose other data.

Implementation scope: System policy separation, injection defenses, tool allowlist, content/privacy controls, redaction and adversarial tests.

Dependencies: M11-04
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Cross-classroom and forbidden-tool attempts fail; sensitive data is absent from telemetry.
<!-- AC:END -->
