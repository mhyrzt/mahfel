---
id: TASK-99
title: M16-01 Platform manifests/config
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
labels: []
milestone: m-16
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 107000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Compose is not a production deployment specification.

Implementation scope: Darkube app/service definitions, images, private/public ports, env/secrets, resources, probes and scrape discovery.

Dependencies: M13, M15
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Manifests validate and contain no plaintext secrets or application persistent volumes.
<!-- AC:END -->
