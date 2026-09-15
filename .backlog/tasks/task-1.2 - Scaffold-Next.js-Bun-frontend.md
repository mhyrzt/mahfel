---
id: TASK-1.2
title: Scaffold Next.js Bun frontend
status: Done
assignee:
  - '@codex'
created_date: '2026-09-07 23:24'
updated_date: '2026-09-08 00:40'
labels: []
milestone: m-0
dependencies: []
parent_task_id: TASK-1
ordinal: 3000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Problem: no independently deployable browser application exists. Scope: initialize Next.js App Router with Bun, strict TypeScript, Tailwind, shadcn/ui, feature-oriented folders, runtime configuration display, and web health endpoint.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 bun.lock and packageManager pin are committed
- [x] #2 Lint, typecheck, Bun tests, and production build pass
- [x] #3 Minimal page renders and reads the configured API base URL
- [x] #4 Next standalone output and shadcn configuration are valid
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Follow the approved Mahfel bootstrap plan for this focused slice; update progress and validation evidence through the Backlog CLI.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Validated with bun install, bun test (1 passing test), bun run lint, bun run typecheck, and bun run build using Next.js 16.3.3.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Created the independently deployable Bun/Next App Router scaffold with strict TypeScript, Tailwind, shadcn/ui configuration, API URL configuration, health route, and standalone output.
<!-- SECTION:FINAL_SUMMARY:END -->
