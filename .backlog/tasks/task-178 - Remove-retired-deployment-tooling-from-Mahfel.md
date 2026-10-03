---
id: TASK-178
title: Remove retired deployment tooling from Mahfel
status: Done
assignee:
  - '@codex'
created_date: '2026-10-03 10:57'
updated_date: '2026-10-03 11:28'
labels: []
dependencies: []
ordinal: 186000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Remove the retired orchestration and secret tooling from this repository only now that deployment will use the existing external Rastploy setup. Preserve application services, Compose development and unrelated edits.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Retired deployment trees, operator CLI and associated CI and Just commands are removed
- [x] #2 Active documentation and roadmap no longer require the retired tooling or link to removed files
- [x] #3 Strict docs, Compose configuration and remaining CI includes validate
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Remove the retired deployment trees and dedicated decisions. 2. Clean root CI, commands, ignores and active docs. 3. Align active production roadmap tracking and validate remaining surfaces.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Removed the retired deployment trees including the operator CLI and host bootstrap, their dedicated ADRs and active references. Kept completed task records as history; renamed the active production milestone and updated its platform task through Backlog CLI. Validated strict MkDocs build, Docker Compose configuration, Just recipe listing, all seven local CI YAML files/includes and git diff whitespace. No other repository changed and no commit or live deployment was performed.

Follow-up Darkube cleanup: confirmed no active source, CI or deployment files refer to Darkube. Replaced the production milestone description through the CLI with a VPS/Rastploy description; CLI replacement assigns m-17 while retaining the M16 title and reassigning its seven tasks. The retired milestone and completed requirements-review task remain as historical records.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Removed retired deployment tooling and active documentation from Mahfel only. Remaining Compose development, image publishing and application boundaries are preserved. Strict docs, Compose configuration, CI includes, Just parsing and whitespace checks pass.
<!-- SECTION:FINAL_SUMMARY:END -->
