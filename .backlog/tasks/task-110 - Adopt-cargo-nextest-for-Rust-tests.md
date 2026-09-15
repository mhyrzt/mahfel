---
id: TASK-110
title: Adopt cargo-nextest for Rust tests
status: Done
assignee:
  - '@codex'
created_date: '2026-09-15 23:24'
updated_date: '2026-09-15 23:37'
labels: []
dependencies: []
ordinal: 118000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Rust test execution should use cargo-nextest for faster, clearer workspace runs while retaining cargo test for doctests that nextest does not execute.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Backend Just workflows run workspace and disposable-database tests through cargo-nextest
- [x] #2 A checked-in nextest profile defines stable local and CI behavior
- [x] #3 The Docker validation target installs a pinned nextest and runs nextest plus Rust doctests
- [x] #4 Developer documentation lists nextest and validation commands parse or pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add a conservative backend nextest profile. 2. Switch backend Just unit/integration workflows to cargo nextest while keeping cargo test --doc. 3. Pin cargo-nextest 0.9.144 in the pre-copy Docker builder layer and use it in the validation target. 4. Update testing prerequisites and commands. 5. Validate configuration and available local gates without pulling application images.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Pinned cargo-nextest 0.9.144, added a checked-in default profile, moved workspace and disposable database test commands to nextest, retained cargo test for doctests, and updated the Docker validation target. A temporary official nextest binary ran 3 affected tests successfully; doctests passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Adopted pinned cargo-nextest for Rust test binaries while retaining explicit doctest coverage in local and container validation workflows.
<!-- SECTION:FINAL_SUMMARY:END -->
