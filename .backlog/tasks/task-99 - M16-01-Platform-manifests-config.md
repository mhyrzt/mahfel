---
id: TASK-99
title: M16-01 Platform deployment/config
status: To Do
assignee: []
created_date: '2026-09-08 00:32'
updated_date: '2026-10-03 11:28'
labels: []
milestone: m-17
dependencies: []
documentation:
  - docs/backlog/roadmap.md
priority: medium
ordinal: 107000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Development configuration is not a production deployment specification. Implementation scope: Production Compose definitions and deployment targets, images, private/public ports, external dependencies, runtime secrets, resources, healthchecks and scrape discovery.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Deployment configuration validates and contains no plaintext secrets; durable application state remains in PostgreSQL and S3.
<!-- AC:END -->
