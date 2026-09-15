---
id: TASK-1
title: Bootstrap Mahfel production foundation
status: In Progress
assignee:
  - '@codex'
created_date: '2026-09-07 23:24'
labels: []
milestone: m-0
dependencies: []
references:
  - ../lms-main/docker-compose.yml
documentation:
  - docs/backlog/roadmap.md
priority: high
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: Mahfel has no implementation foundation, runtime boundaries, development stack, architecture record, or execution backlog. Scope: establish the repository, compileable service shells, frontend, production images, local infrastructure, observability, documentation, ADRs, detailed Backlog.md roadmap, validation, and a clean local commit without implementing product features.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Repository contains exactly four deployable application components: mahfel-web, mahfel-api, mahfel-media, and mahfel-worker
- [ ] #2 Rust workspace, frontend, documentation, and all four application images build successfully
- [ ] #3 One command starts the complete local stack and Prometheus scrapes all three Rust services
- [ ] #4 Architecture documentation, ADR 0001 through 0014, and detailed milestones M0 through M16 exist
- [ ] #5 All requested quality gates and live health checks are run with exact results recorded
- [ ] #6 Repository finishes on main with a local bootstrap commit, no push, and a clean working tree
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Register milestones and task hierarchy. 2. Scaffold Rust workspace and service contracts. 3. Scaffold Next.js/Bun frontend. 4. Add images, Compose, configuration, observability, and health checks. 5. Build MkDocs architecture, ADRs, and mirrored roadmap. 6. Run every required static, build, Compose, and live-stack check. 7. Finalize Backlog tasks and create one local Conventional Commit.
<!-- SECTION:PLAN:END -->
