---
id: TASK-111
title: Strengthen frontend lint policy
status: Done
assignee:
  - '@codex'
created_date: '2026-09-15 23:28'
updated_date: '2026-09-15 23:37'
labels: []
dependencies: []
ordinal: 119000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Frontend code should fail fast on unsafe or inconsistent JavaScript/TypeScript patterns and source comments should be prohibited except explicit ESLint disable directives.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 ESLint treats explicit any, console/debugger use, inconsistent type imports, unused variables, loose equality, and avoidable control-flow/style hazards as errors
- [x] #2 A local lint rule rejects line and block comments unless they are ESLint disable directives
- [x] #3 Generated Next.js declaration files remain ignored and existing frontend source passes the strict lint configuration
- [x] #4 Frontend format, lint, and typecheck Just recipes pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add a local ESLint rule that inspects all comments and permits only eslint-disable directives. 2. Enable a focused strict baseline across JavaScript, TypeScript, React, imports, control flow, and debugging output without requiring type-aware ESLint project services. 3. Run Prettier, ESLint, and TypeScript checks through the frontend Justfile and adjust only genuine violations. 4. Document the comment rule in frontend conventions and commit it with the developer workflow changes.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Added a local ESLint comment rule that permits only eslint-disable directives, enables unused-disable reporting, and applies a stricter TypeScript/React/control-flow baseline. Pinned Prettier 3.9.6 and verified frozen install, format, lint, typecheck, and test.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Strengthened frontend linting, prohibited ordinary source comments, and added pinned Prettier formatting with passing component workflows.
<!-- SECTION:FINAL_SUMMARY:END -->
