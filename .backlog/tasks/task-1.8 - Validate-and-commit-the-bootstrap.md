---
id: TASK-1.8
title: Validate and commit the bootstrap
status: In Progress
assignee: []
created_date: '2026-09-07 23:24'
updated_date: '2026-09-15 02:04'
labels: []
milestone: m-0
dependencies: []
parent_task_id: TASK-1
ordinal: 9000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: scaffolding is not complete until reproducibility and runtime behavior are demonstrated. Scope: run all Rust, Bun, docs, Docker, live health, scrape, and datasource checks; record evidence; finalize M0; create a local commit.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Every required command is run and failures are fixed or reported exactly
- [x] #2 Complete local stack health is checked
- [x] #3 Prometheus target state and Grafana datasource are verified
- [x] #4 Diff contains no secrets or unrelated artifacts
- [ ] #5 Local commit exists on main, no push occurs, and worktree is clean
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Follow the approved Mahfel bootstrap plan for this focused slice; update progress and validation evidence through the Backlog CLI.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Validation completed: all four images built; full Rust validation target passed; live Compose health and observability passed; strict docs and Compose config passed. Exact host Rust native gates fail on Fedora because mediasoup-sys receives an empty libstdc++ search path; exact host Next build fails because this sandbox denies Turbopack worker port binding, while the same build passed in Docker.
<!-- SECTION:NOTES:END -->
